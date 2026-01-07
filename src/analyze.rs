use std::collections::HashMap;

use crate::{
    intrinsics::get_intrinsic_arity,
    lang::{Arity, ArityCombineError, Block, Branch, Loop, Term, Type},
    program::{NamespaceId, Program},
};

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisError {
    IndefiniteSize,
    IncompatibleTypes,
    Pending,
    MissingDeclaration(String),
}

impl From<ArityCombineError> for AnalysisError {
    fn from(value: ArityCombineError) -> Self {
        match value {
            ArityCombineError::DifferingSizes => Self::IndefiniteSize,
            ArityCombineError::IncompatibleTypes => Self::IncompatibleTypes,
        }
    }
}

pub type BlockAnalysisResult = Result<Arity, AnalysisError>;

pub type NamespaceArities = HashMap<String, BlockAnalysisResult>;

pub type AritiesByNamespace = Vec<NamespaceArities>;

pub struct Analysis<'a> {
    pub arities: AritiesByNamespace,
    pub namespace: NamespaceId,
    pub program: &'a Program,
}

enum BlockTruthiness {
    AlwaysTruthy,
    AlwaysFalsy,
    Unknown,
}

fn block_is_always_truthy(b: &Block) -> BlockTruthiness {
    let Some(last) = b.terms.last() else {
        return BlockTruthiness::Unknown;
    };

    let is_truthy: bool = match last {
        Term::String(t) => !t.is_empty(),
        Term::Number(t) => !t.is_nan() && *t != 0f64,
        Term::Bool(true) | Term::Address(_) => true,
        Term::Bool(false) => false,
        Term::Capture(..) | Term::Name(_, _) | Term::Branch(_) | Term::Loop(_) => {
            return BlockTruthiness::Unknown;
        }
    };

    if is_truthy {
        BlockTruthiness::AlwaysTruthy
    } else {
        BlockTruthiness::AlwaysFalsy
    }
}

fn into_isolated(a: Arity) -> Result<Arity, AnalysisError> {
    if !a.captures.waiting.is_empty() {
        return Err(AnalysisError::MissingDeclaration(
            a.captures.waiting.join(", "),
        ));
    }
    Ok(Arity {
        pushes: a.pushes,
        pops: a.pops,
        captures: Default::default(),
    })
}

pub fn analyze_condition<'a>(analysis: &mut Analysis<'a>, b: &'a Block) -> BlockAnalysisResult {
    let mut res = analyze_block(analysis, b)?;
    res.pop_any();
    Ok(res)
}

pub fn analyze_block<'a>(analysis: &mut Analysis<'a>, b: &'a Block) -> BlockAnalysisResult {
    let mut a = Arity::noop();
    for term in &b.terms {
        a = Arity::serial(&a, &analyze_term(analysis, term)?)?;
    }
    Ok(a)
}

fn analyze_name(analysis: &mut Analysis, n: &str) -> BlockAnalysisResult {
    if let Some(arity) = get_intrinsic_arity(n)? {
        return Ok(arity.clone());
    }

    let Some((resolved_namespace_id, resolved_name)) =
        analysis.program.resolve_function(analysis.namespace, n)
    else {
        return Ok(Arity::recall(n.to_owned()));
    };

    let Some(Some(arity_result)) = analysis
        .arities
        .get(resolved_namespace_id)
        .map(|e| e.get(resolved_name))
    else {
        return Err(AnalysisError::Pending);
    };
    arity_result.clone()
}

fn analyze_branch<'a>(analysis: &mut Analysis<'a>, branch: &'a Branch) -> BlockAnalysisResult {
    let mut running = Arity::noop();

    let mut combined: Option<Arity> = None;
    let mut add_termination = |a: Arity| -> Result<(), AnalysisError> {
        combined = Some(if let Some(before) = combined.take() {
            match Arity::parallel(&before, &a) {
                Ok(n) => n,
                Err(ArityCombineError::DifferingSizes) => {
                    return Err(AnalysisError::IndefiniteSize);
                }
                Err(ArityCombineError::IncompatibleTypes) => {
                    return Err(AnalysisError::IncompatibleTypes);
                }
            }
        } else {
            a
        });

        Ok(())
    };

    for arm in &branch.arms {
        let condition_arity = analyze_condition(analysis, &arm.0)?;
        running = Arity::serial(&running, &condition_arity)?;

        let (possible, last_arm) = match block_is_always_truthy(&arm.0) {
            BlockTruthiness::AlwaysTruthy => (true, true),
            BlockTruthiness::AlwaysFalsy => (false, false),
            BlockTruthiness::Unknown => (true, false),
        };

        if possible {
            let block_arity = analyze_block(analysis, &arm.1)?;
            let arity = Arity::serial(&running, &block_arity)?;
            add_termination(arity)?;
        }

        if last_arm {
            return Ok(combined.expect("Unable to combine last branch arm"));
        }
    }

    add_termination(running)?;

    Ok(combined.expect("Unable to combine branch arms"))
}

fn analyze_loop<'a>(analysis: &mut Analysis<'a>, loop_v: &'a Loop) -> BlockAnalysisResult {
    let pre_arity = if let Some(pre) = &loop_v.pre_condition {
        Some(analyze_condition(analysis, pre)?)
    } else {
        None
    };

    let main_arity = analyze_block(analysis, &loop_v.body)?;

    let post_arity = if let Some(post) = &loop_v.post_condition {
        Some(analyze_condition(analysis, post)?)
    } else {
        None
    };

    if pre_arity.is_none() && post_arity.is_none() {
        return Err(AnalysisError::IndefiniteSize);
    }

    let mut running_arity = Arity::noop();
    let mut possible_arity = None;
    let mut seen_states = vec![];

    let record_next_exit_arity = |running: &mut Arity,
                                  possible: &mut Option<Arity>,
                                  next: &Arity|
     -> Result<(), AnalysisError> {
        *running = Arity::serial(running, next)?;

        let next_possible = match possible {
            Some(possible) => Arity::parallel(possible, running)?,
            None => running.clone(),
        };

        *possible = Some(next_possible);

        Ok(())
    };

    loop {
        if seen_states.contains(&running_arity) {
            break;
        }
        seen_states.push(running_arity.clone());

        if let Some(pre) = pre_arity.as_ref() {
            record_next_exit_arity(&mut running_arity, &mut possible_arity, pre)?;
        }

        running_arity = Arity::serial(&running_arity, &main_arity)?;

        if let Some(post) = post_arity.as_ref() {
            record_next_exit_arity(&mut running_arity, &mut possible_arity, post)?;
        }
    }

    Ok(possible_arity.expect("Must have filled possible_arity at least once"))
}

pub fn analyze_term<'a>(analysis: &mut Analysis<'a>, term: &'a Term) -> BlockAnalysisResult {
    match term {
        Term::String(_) => Ok(Arity::literal(Type::String)),
        Term::Number(_) => Ok(Arity::literal(Type::Number)),
        Term::Bool(_) => Ok(Arity::literal(Type::Bool)),
        Term::Address(_) => Ok(Arity::literal(Type::Address)),
        Term::Name(n, _) => analyze_name(analysis, n.as_str()),
        Term::Branch(branch) => analyze_branch(analysis, branch),
        Term::Loop(loop_v) => analyze_loop(analysis, loop_v),
        Term::Capture(n, ..) => Ok(Arity::capture(n.to_owned())),
    }
}

fn get_arity_at(by_namespace: &mut AritiesByNamespace, i: NamespaceId) -> &mut NamespaceArities {
    while by_namespace.len() <= i {
        by_namespace.push(NamespaceArities::new());
    }

    &mut by_namespace[i]
}

pub fn analyze_program(program: &Program) -> AritiesByNamespace {
    let mut analysis = Analysis {
        arities: AritiesByNamespace::new(),
        namespace: 0,
        program,
    };

    loop {
        let mut resolved_something = false;

        for (i, namespace) in program.namespaces.iter().enumerate() {
            analysis.namespace = i;
            for func in namespace.functions.values() {
                if get_arity_at(&mut analysis.arities, i).contains_key(&func.name) {
                    continue;
                }
                match analyze_block(&mut analysis, &func.body)
                    .map(|e| into_isolated(e))
                    .flatten()
                {
                    Err(AnalysisError::Pending) => {}
                    e => {
                        resolved_something = true;
                        get_arity_at(&mut analysis.arities, i).insert(func.name.clone(), e);
                    }
                }
            }
        }

        if !resolved_something {
            break;
        }
    }

    analysis.arities
}

pub fn analyze_block_in_namespace(
    arities: &AritiesByNamespace,
    namespace: NamespaceId,
    block: &Block,
    program: &Program,
) -> BlockAnalysisResult {
    let mut analysis = Analysis {
        arities: arities.clone(),
        namespace,
        program,
    };

    analyze_block(&mut analysis, block)
}
