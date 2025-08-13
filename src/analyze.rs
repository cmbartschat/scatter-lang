use std::collections::HashMap;

use crate::{
    intrinsics::get_intrinsic,
    lang::{Arity, ArityCombineError, Block, Module, Term, Type, Value},
};

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisError {
    IndefiniteSize,
    IncompatibleTypes,
    Pending,
}

pub type BlockAnalysisResult = Result<Arity, AnalysisError>;

pub type Arities = HashMap<String, BlockAnalysisResult>;

#[derive(Debug)]
pub struct Analysis {
    pub arities: Arities,
    pub body_arity: BlockAnalysisResult,
}

fn block_is_always_truthy(b: &Block) -> Option<bool> {
    match (b.terms.len(), b.terms.first()) {
        (1, Some(Term::Literal(t))) => Some(t.is_truthy()),
        _ => None,
    }
}

pub fn analyze_condition(arities: &Arities, b: &Block) -> BlockAnalysisResult {
    Ok(analyze_block(arities, b)?.with_pop(Type::Unknown))
}

pub fn analyze_block(arities: &Arities, b: &Block) -> BlockAnalysisResult {
    let mut a = Arity::noop();
    for term in b.terms.iter() {
        a.serial(&analyze_term(arities, term)?);
    }
    Ok(a)
}

pub fn analyze_term(arities: &Arities, term: &Term) -> BlockAnalysisResult {
    match term {
        Term::Literal(t) => Ok(match t {
            Value::String(_) => Arity::literal(Type::String),
            Value::Number(_) => Arity::literal(Type::Number),
            Value::Bool(_) => Arity::literal(Type::Bool),
        }),
        Term::Name(n) => {
            if let Some(a) = get_intrinsic(n) {
                return Ok(a.arity.clone());
            }
            match arities.get(n) {
                Some(Ok(a)) => Ok(a.clone()),
                None | Some(Err(AnalysisError::Pending)) => Err(AnalysisError::Pending),
                Some(e) => e.clone(),
            }
        }
        Term::Branch(branch) => {
            let mut running = Arity::noop();

            let mut combined: Option<Arity> = None;
            let mut add_termination = |a: Arity| -> Result<(), AnalysisError> {
                combined = Some(if let Some(before) = combined.take() {
                    match Arity::parallel(&before, &a) {
                        Ok(n) => n,
                        Err(ArityCombineError::DifferingSizes) => {
                            return Err(AnalysisError::IndefiniteSize);
                        }
                        Err(ArityCombineError::DifferentInputs) => {
                            return Err(AnalysisError::IncompatibleTypes);
                        }
                    }
                } else {
                    a
                });

                Ok(())
            };

            for arm in branch.arms.iter() {
                let condition_arity = analyze_condition(arities, &arm.0)?;
                running.serial(&condition_arity);

                let (possible, last_arm) = match block_is_always_truthy(&arm.0) {
                    Some(true) => (true, true),
                    Some(false) => (false, false),
                    None => (true, false),
                };

                if possible {
                    let block_arity = analyze_block(arities, &arm.1)?;
                    let arity = running.clone().with_serial(&block_arity);
                    add_termination(arity)?;
                }

                if last_arm {
                    return Ok(combined.unwrap());
                }
            }

            add_termination(running)?;

            Ok(combined.unwrap())
        }
        Term::Loop(loop_v) => {
            let pre_arity = if let Some(pre) = &loop_v.pre_condition {
                analyze_condition(arities, pre)?
            } else {
                Arity::noop()
            };

            if !pre_arity.is_neutral() {
                return BlockAnalysisResult::Err(AnalysisError::IndefiniteSize);
            }

            let main_arity = if let Some(post) = &loop_v.post_condition {
                analyze_block(arities, &loop_v.body)?
                    .with_serial(&analyze_condition(arities, post)?)
            } else {
                analyze_block(arities, &loop_v.body)?
            };

            if !main_arity.is_neutral() {
                return BlockAnalysisResult::Err(AnalysisError::IndefiniteSize);
            }

            let mut arity = pre_arity.clone();

            let mut alternates = vec![];

            arity.serial(&main_arity);
            alternates.push(arity.clone());

            arity.serial(&pre_arity);
            alternates.push(arity.clone());

            arity.serial(&main_arity);
            alternates.push(arity.clone());

            arity.serial(&pre_arity);
            alternates.push(arity.clone());

            arity.serial(&main_arity);
            for alt in alternates {
                arity = match Arity::parallel(&arity, &alt) {
                    Ok(n) => n,
                    Err(ArityCombineError::DifferingSizes) => {
                        return Err(AnalysisError::IndefiniteSize);
                    }
                    Err(ArityCombineError::DifferentInputs) => {
                        return Err(AnalysisError::IncompatibleTypes);
                    }
                }
            }
            Ok(arity)
        }
    }
}

pub fn analyze(m: &Module) -> Analysis {
    let mut arities = HashMap::new();

    loop {
        let mut resolved_something = false;
        for func in m.functions.iter() {
            if arities.contains_key(&func.name) {
                continue;
            }
            match analyze_block(&arities, &func.body) {
                Err(AnalysisError::Pending) => {}
                e => {
                    resolved_something = true;
                    arities.insert(func.name.to_owned(), e);
                }
            }
        }

        if !resolved_something {
            break;
        }
    }

    let body_arity = analyze_block(&arities, &m.body);

    Analysis {
        arities,
        body_arity,
    }
}
