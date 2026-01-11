use std::{collections::HashMap, fmt::Write as _};

use crate::{analyze::AnalysisError, lang::Type};

type Index = usize;

#[derive(Clone, PartialEq, Debug)]
pub struct MultiIndex {
    pub el: Index,
    pub next: Option<Box<MultiIndex>>,
}

impl MultiIndex {
    pub fn contains(&self, i: usize) -> bool {
        self.el == i || self.next.as_ref().is_some_and(|f| f.contains(i))
    }

    pub fn insert(&mut self, i: usize) {
        match self.el.cmp(&i) {
            std::cmp::Ordering::Greater => {
                let prev_el = self.el;
                self.el = i;
                self.insert(prev_el);
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Less => match &mut self.next {
                Some(n) => n.insert(i),
                None => {
                    self.next = Some(Box::new(Self { el: i, next: None }));
                }
            },
        }
    }

    pub fn iter(&self) -> MultiIndexIter<'_> {
        MultiIndexIter::new(Some(self))
    }

    pub fn iter_rest(&self) -> MultiIndexIter<'_> {
        MultiIndexIter::new(self.next.as_deref())
    }
}

impl From<(usize, usize)> for MultiIndex {
    fn from(value: (usize, usize)) -> Self {
        Self {
            el: value.0,
            next: Some(Box::new(Self {
                el: value.1,
                next: None,
            })),
        }
    }
}

impl From<usize> for MultiIndex {
    fn from(value: usize) -> Self {
        Self {
            el: value,
            next: None,
        }
    }
}

pub struct MultiIndexIter<'a> {
    next: Option<&'a MultiIndex>,
}

impl<'a> MultiIndexIter<'a> {
    pub fn new(target: Option<&'a MultiIndex>) -> Self {
        Self { next: target }
    }
}

impl Iterator for MultiIndexIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(c) => {
                let res = c.el;
                self.next = c.next.as_deref();
                Some(res)
            }
            None => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ResultantType {
    Normal(Type),
    Dependent(MultiIndex),
    Recall(String),
}

impl From<Type> for ResultantType {
    fn from(value: Type) -> Self {
        Self::Normal(value)
    }
}

impl ResultantType {
    pub fn stringify(&self) -> String {
        match self {
            ResultantType::Normal(t) => t.stringify().into(),
            ResultantType::Dependent(d) => {
                let mut str = String::new();
                for t in d.iter() {
                    if !str.is_empty() {
                        str.push('|');
                    }
                    write!(&mut str, "{t}").expect("Write error in ResultantType::stringify");
                }
                str
            }
            _ => todo!(),
        }
    }

    pub fn parse(source: &str) -> Option<Self> {
        if let Some(t) = Type::parse_raw(source) {
            return Some(Self::Normal(t));
        }
        let mut segments = source.split('|').map(|f| f.parse::<usize>().ok());
        let first = segments.next()??;
        let mut indices = MultiIndex::from(first);
        for segment in segments {
            indices.insert(segment?);
        }
        Some(Self::Dependent(indices))
    }

    pub fn references(&self, i: usize) -> bool {
        match self {
            ResultantType::Normal(_) => false,
            ResultantType::Dependent(d) => d.contains(i),
            _ => todo!(),
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        match (self, other) {
            (ResultantType::Normal(a), ResultantType::Normal(b)) => a.union(*b).into(),
            (ResultantType::Dependent(_), ResultantType::Normal(n))
            | (ResultantType::Normal(n), ResultantType::Dependent(_)) => n.to_owned().into(),
            (ResultantType::Dependent(s), ResultantType::Dependent(other)) => {
                ResultantType::Dependent(other.iter().fold(s.clone(), |mut a, f| {
                    a.insert(f);
                    a
                }))
            }
            _ => todo!(),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EffectCertainty {
    Always,
    Sometimes,
}

#[derive(Clone, PartialEq)]
pub struct VariableEffect {
    pub expects: bool,
    pub defines: Option<(ResultantType, EffectCertainty)>,
}
impl VariableEffect {
    fn serial(&mut self, f: &VariableEffect) {
        if f.expects
            && self
                .defines
                .as_ref()
                .is_none_or(|f| f.1 != EffectCertainty::Always)
        {
            self.expects = true;
        }

        if let Some(d2) = &f.defines {
            if let Some(d1) = self.defines.as_mut() {
                if d2.1 == EffectCertainty::Always {
                    *d1 = d2.clone();
                } else {
                    d1.0 = d1.0.union(&d2.0);
                }
            } else {
                self.defines = Some(d2.clone());
            }
        }
    }

    fn parallel(&mut self, right: &Self) {
        self.expects = self.expects || right.expects;

        match (&mut self.defines, &right.defines) {
            (None, None) => {}
            (None, Some(d)) => self.defines = Some((d.0.clone(), EffectCertainty::Sometimes)),
            (Some(d), None) => d.1 = EffectCertainty::Sometimes,
            (Some((d1, EffectCertainty::Always)), Some((d2, EffectCertainty::Always))) => {
                self.defines = Some((d1.union(d2), EffectCertainty::Always));
            }
            (Some((d1, _)), Some((d2, _))) => {
                self.defines = Some((d1.union(d2), EffectCertainty::Sometimes));
            }
        }
    }

    pub fn references(&self, i: usize) -> bool {
        self.defines.as_ref().is_some_and(|f| f.0.references(i))
    }

    fn maybe(&mut self) {
        if let Some(d) = &mut self.defines {
            d.1 = EffectCertainty::Sometimes;
        }
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct CaptureEffects {
    pub variables: HashMap<String, VariableEffect>,
}

impl CaptureEffects {
    pub fn defines(v: String, t: ResultantType) -> Self {
        let mut res = Self::default();

        res.variables.insert(
            v,
            VariableEffect {
                expects: false,
                defines: Some((t, EffectCertainty::Always)),
            },
        );
        res
    }

    pub fn expects(v: String) -> Self {
        let mut res = Self::default();

        res.variables.insert(
            v,
            VariableEffect {
                expects: true,
                defines: None,
            },
        );
        res
    }

    pub fn serial(&mut self, right: &Self) {
        right.variables.iter().for_each(|(name, right_effect)| {
            if let Some(f) = self.variables.get_mut(name) {
                f.serial(right_effect);
            }
        });
    }

    pub fn parallel(&mut self, right: &Self) {
        self.variables.iter_mut().for_each(|(name, f)| {
            if let Some(right_effect) = right.variables.get(name) {
                f.parallel(right_effect);
            } else {
                f.maybe();
            }
        });

        right.variables.iter().for_each(|(name, f)| {
            if (!self.variables.contains_key(name)) {
                let mut s = f.clone();
                s.maybe();
                self.variables.insert(name.clone(), s);
            }
        });
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct Arity {
    pub pops: Vec<Type>,
    pub pushes: Vec<ResultantType>,
    pub captures: CaptureEffects,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ArityCombineError {
    DifferingSizes,
    IncompatibleTypes,
}

impl Arity {
    pub fn noop() -> Self {
        Self::default()
    }

    pub fn capture(name: String) -> Self {
        Self {
            pops: vec![Type::Unknown],
            pushes: vec![],
            captures: CaptureEffects::defines(name, ResultantType::Dependent(0.into())),
        }
    }

    pub fn recall(name: String) -> Self {
        Self {
            pops: vec![],
            pushes: vec![ResultantType::Recall(name.clone())],
            captures: CaptureEffects::expects(name),
        }
    }

    pub fn literal(r: Type) -> Self {
        Self {
            pushes: vec![r.into()],
            ..Self::default()
        }
    }

    pub fn unary(a: Type, r: Type) -> Self {
        Self {
            pops: vec![a],
            pushes: vec![r.into()],
            ..Self::default()
        }
    }

    pub fn push_two(a: Type, b: Type) -> Self {
        Self {
            pops: vec![],
            pushes: vec![a.into(), b.into()],
            ..Self::default()
        }
    }

    pub fn pop_two(a: Type, b: Type) -> Self {
        Self {
            pops: vec![b, a],
            ..Self::default()
        }
    }

    pub fn binary(a: Type, b: Type, r: Type) -> Self {
        Self {
            pops: vec![a, b],
            pushes: vec![r.into()],
            ..Self::default()
        }
    }

    pub fn generic_1<T>(pop_count: usize, res1: T) -> Self
    where
        T: Into<MultiIndex>,
    {
        let mut res = Arity::noop();
        for _ in 0..pop_count {
            res.pops.push(Type::Unknown);
        }
        res.pushes.push(ResultantType::Dependent(res1.into()));
        res
    }

    pub fn generic_2<T1, T2>(pop_count: usize, res1: T1, res2: T2) -> Self
    where
        T1: Into<MultiIndex>,
        T2: Into<MultiIndex>,
    {
        let mut res = Arity::noop();
        for _ in 0..pop_count {
            res.pops.push(Type::Unknown);
        }
        res.pushes.push(ResultantType::Dependent(res1.into()));
        res.pushes.push(ResultantType::Dependent(res2.into()));
        res
    }

    pub fn generic_3<T1, T2, T3>(pop_count: usize, res1: T1, res2: T2, res3: T3) -> Self
    where
        T1: Into<MultiIndex>,
        T2: Into<MultiIndex>,
        T3: Into<MultiIndex>,
    {
        let mut res = Arity::noop();
        for _ in 0..pop_count {
            res.pops.push(Type::Unknown);
        }
        res.pushes.push(ResultantType::Dependent(res1.into()));
        res.pushes.push(ResultantType::Dependent(res2.into()));
        res.pushes.push(ResultantType::Dependent(res3.into()));
        res
    }

    pub fn number_binary() -> Self {
        Self::binary(Type::Number, Type::Number, Type::Number)
    }

    pub fn number_unary() -> Self {
        Self::unary(Type::Number, Type::Number)
    }

    pub fn size(&self) -> (usize, usize) {
        (self.pops.len(), self.pushes.len())
    }

    pub fn pop_any(&mut self) {
        if self.pushes.pop().is_none() {
            self.pops.push(Type::Unknown);
        }
    }

    pub fn attempt_pop(&mut self, term: Type) -> Result<ResultantType, ArityCombineError> {
        match (self.pushes.pop(), term) {
            (Some(ResultantType::Normal(t)), _) => {
                if !t.assignable_to(term) {
                    return Err(ArityCombineError::IncompatibleTypes);
                }
                Ok(t.into())
            }
            (None, _) => {
                self.pops.push(term);
                Ok(ResultantType::Dependent((self.pops.len() - 1).into()))
            }
            (Some(ResultantType::Dependent(i)), Type::Unknown) => Ok(ResultantType::Dependent(i)),
            (Some(ResultantType::Dependent(i)), _) => {
                for x in i.iter() {
                    if term.assignable_to(self.pops[x]) {
                        for push in &mut self.pushes {
                            if push.references(x) {
                                *push = ResultantType::Normal(term);
                            }
                        }
                        self.pops[x] = term;
                    } else {
                        return Err(ArityCombineError::IncompatibleTypes);
                    }
                }
                Ok(term.into())
            }
            _ => todo!(),
        }
    }

    pub fn push<T>(&mut self, term: T)
    where
        T: Into<ResultantType>,
    {
        self.pushes.push(term.into());
    }

    pub fn serial(first: &Arity, second: &Arity) -> Result<Arity, ArityCombineError> {
        let mut running = first.clone();
        let resolved_pop_types = second.pops.iter().try_fold(vec![], |mut acc, f| {
            acc.push(running.attempt_pop(f.to_owned())?);
            Ok(acc)
        })?;

        second.pushes.iter().for_each(|f| match f {
            ResultantType::Normal(t) => running.push(*t),
            ResultantType::Dependent(x) => {
                let mut first = resolved_pop_types[x.el].clone();
                let others = x.iter_rest().map(|f| &resolved_pop_types[f]);
                for other in others {
                    first = first.union(other);
                }
                running.push(first);
            }
            _ => todo!(),
        });

        running.captures.serial(&second.captures);

        Ok(running)
    }

    pub fn stringify(&self) -> String {
        let mut res = String::new();
        for (i, pop) in self.pops.iter().enumerate().rev() {
            let should_show_digit = pop == &Type::Unknown
                && (self.pushes.iter().any(|f| f.references(i))
                    || self.captures.variables.iter().any(|f| f.1.references(i)));
            if should_show_digit {
                write!(&mut res, "{i}").expect("Write error in Arity::stringify");
            } else {
                res.push_str(pop.stringify());
            }
            res.push(' ');
        }

        res.push('-');

        for push in &self.pushes {
            res.push(' ');
            res.push_str(&push.stringify());
        }

        self.captures.variables.iter().for_each(|(name, defs)| {
            res.push_str(", ");
            if defs.expects {
                res.push('>');
            }
            res.push_str(name);
            if let Some(def) = &defs.defines {
                res.push(':');
                res.push_str(def.0.stringify().as_ref());
                match def.1 {
                    EffectCertainty::Always => {}
                    EffectCertainty::Sometimes => res.push('?'),
                }
            }
        });

        res
    }

    pub fn extend_pops(&mut self) {
        self.pops.push(Type::Unknown);
        self.pushes
            .insert(0, ResultantType::Dependent((self.pops.len() - 1).into()));
    }

    fn resolve_dependents(pushes: &mut Vec<ResultantType>, pops: &[Type]) {
        for push in pushes {
            match push {
                ResultantType::Normal(_) => {}
                ResultantType::Dependent(multi_index) => {
                    let mut resolved_type: Option<Type> = None;
                    for index in multi_index.iter() {
                        if pops[index] != Type::Unknown {
                            resolved_type = match resolved_type {
                                Some(t) => t.inter(pops[index]),
                                None => Some(pops[index]),
                            }
                        }
                    }
                    if let Some(resolved_type) = resolved_type {
                        *push = resolved_type.into();
                    }
                }
                _ => todo!(),
            }
        }
    }

    pub fn parallel(raw_left: &Arity, raw_right: &Arity) -> Result<Arity, ArityCombineError> {
        let mut left = raw_left.clone();
        let mut right = raw_right.clone();
        let left_pops = right.pops.len().saturating_sub(left.pops.len());
        let right_pops = left.pops.len().saturating_sub(right.pops.len());
        for _ in 0..left_pops {
            left.extend_pops();
        }

        for _ in 0..right_pops {
            right.extend_pops();
        }

        if left.size() != right.size() {
            return Err(ArityCombineError::DifferingSizes);
        }

        let mut res = Arity::noop();

        for (i, t) in left.pops.iter().enumerate() {
            let Some(expected_type) = right.pops[i].inter(*t) else {
                return Err(ArityCombineError::IncompatibleTypes);
            };
            res.pops.push(expected_type);
        }

        Self::resolve_dependents(&mut left.pushes, &res.pops);
        Self::resolve_dependents(&mut right.pushes, &res.pops);

        for (i, t) in left.pushes.iter().enumerate() {
            res.pushes.push(t.union(&right.pushes[i]));
        }

        Ok(res)
    }

    #[allow(dead_code)]
    pub fn parse(source: &str) -> Option<Self> {
        let mut comma_separated = source.split(',').map(str::trim);
        let (pops, pushes) = comma_separated.next()?.split_once('-')?;
        let pops = pops
            .split(' ')
            .rev()
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .try_fold(vec![], |mut acc, e| {
                acc.push(Type::parse_as_pop(e)?);
                Some(acc)
            })?;

        let pushes = pushes
            .split(' ')
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .try_fold(vec![], |mut acc, e| {
                acc.push(ResultantType::parse(e)?);
                Some(acc)
            })?;

        let mut captures = CaptureEffects::default();

        for mut section in comma_separated {
            let expects = if section.starts_with('>') {
                section = &section[1..];
                true
            } else {
                false
            };

            if let Some((name, mut rest)) = section.split_once(':') {
                let frequency = if rest.ends_with('?') {
                    rest = &rest[0..(rest.len().saturating_sub(1))];
                    EffectCertainty::Sometimes
                } else {
                    EffectCertainty::Always
                };

                let t = ResultantType::parse(rest.trim())?;

                captures.variables.insert(
                    name.to_owned(),
                    VariableEffect {
                        expects,
                        defines: Some((t, frequency)),
                    },
                );
            } else {
                captures.variables.insert(
                    section.to_owned(),
                    VariableEffect {
                        expects,
                        defines: None,
                    },
                );
            }
        }

        Some(Self {
            pops,
            pushes,
            captures,
        })
    }
}

impl std::fmt::Debug for Arity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", &self.stringify())
    }
}

impl From<(Vec<Type>, Vec<Type>)> for Arity {
    fn from(value: (Vec<Type>, Vec<Type>)) -> Self {
        Self {
            pops: value.0,
            pushes: value.1.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }
}
