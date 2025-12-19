use std::fmt::Write;

use crate::lang::Type;

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
        }
    }

    pub fn references(&self, i: usize) -> bool {
        match self {
            ResultantType::Normal(_) => false,
            ResultantType::Dependent(d) => d.contains(i),
        }
    }

    pub fn union(&self, other: &Self) -> Result<Self, ()> {
        match (self, other) {
            (ResultantType::Normal(a), ResultantType::Normal(b)) => Ok(a.union(*b).into()),
            (ResultantType::Dependent(_), ResultantType::Normal(n))
            | (ResultantType::Normal(n), ResultantType::Dependent(_)) => Ok(n.to_owned().into()),
            (ResultantType::Dependent(s), ResultantType::Dependent(other)) => Ok(
                ResultantType::Dependent(other.iter().fold(s.clone(), |mut a, f| {
                    a.insert(f);
                    a
                })),
            ),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Arity {
    pub pops: Vec<Type>,
    pub pushes: Vec<ResultantType>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ArityCombineError {
    DifferingSizes,
    DifferentInputs,
}

impl Arity {
    pub fn noop() -> Self {
        Self {
            pops: vec![],
            pushes: vec![],
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.pops.len(), self.pushes.len())
    }

    pub fn literal(r: Type) -> Self {
        Self::noop().with_push(r)
    }

    pub fn unary(a: Type, r: Type) -> Self {
        Self::noop().with_pop(a).with_push(r)
    }

    pub fn push_two(a: Type, b: Type) -> Self {
        Self::noop().with_push(a).with_push(b)
    }

    pub fn binary(a: Type, b: Type, r: Type) -> Self {
        Self::noop().with_pop(a).with_pop(b).with_push(r)
    }

    pub fn in_out(pop_count: usize, push_count: usize) -> Self {
        let mut r = Self::noop();
        for _ in 0..pop_count {
            r.pop(Type::Unknown);
        }

        for _ in 0..push_count {
            r.push(Type::Unknown);
        }
        r
    }

    pub fn generic_1<T>(pop_count: usize, res1: T) -> Self
    where
        T: Into<MultiIndex>,
    {
        let mut res = Arity::noop();
        for _ in 0..pop_count {
            res.pop(Type::Unknown);
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
            res.pop(Type::Unknown);
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
            res.pop(Type::Unknown);
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

    pub fn pop(&mut self, term: Type) -> ResultantType {
        match (self.pushes.pop(), term) {
            (Some(ResultantType::Normal(t)), _) => t.into(),
            (None, _) => {
                self.pops.push(term);
                ResultantType::Dependent((self.pops.len() - 1).into())
            }
            (Some(ResultantType::Dependent(i)), Type::Unknown) => ResultantType::Dependent(i),
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
                        todo!("Handle incompatible types")
                    }
                }
                term.into()
            }
        }
    }

    pub fn push<T>(&mut self, term: T)
    where
        T: Into<ResultantType>,
    {
        self.pushes.push(term.into());
    }

    pub fn with_pop(mut self, other: Type) -> Arity {
        self.pop(other);
        self
    }

    pub fn with_push(mut self, other: Type) -> Arity {
        self.push(other);
        self
    }

    pub fn serial(&mut self, other: &Arity) {
        let mapped_types = other
            .pops
            .iter()
            .map(|f| self.pop(f.to_owned()))
            .collect::<Vec<ResultantType>>();

        other.pushes.iter().for_each(|f| match f {
            ResultantType::Normal(t) => self.push(*t),
            ResultantType::Dependent(x) => {
                let mut first = mapped_types[x.el].clone();
                let others = x.iter_rest().map(|f| &mapped_types[f]);
                for other in others {
                    first = first.union(other).unwrap();
                }
                self.push(first);
            }
        });
    }

    pub fn stringify(&self) -> String {
        let mut res = String::new();
        for (i, pop) in self.pops.iter().enumerate().rev() {
            if pop == &Type::Unknown && self.pushes.iter().any(|f| f.references(i)) {
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

        res
    }

    pub fn extend_pops(&mut self) {
        self.pops.push(Type::Unknown);
        self.pushes
            .insert(0, ResultantType::Dependent((self.pops.len() - 1).into()));
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
                return Err(ArityCombineError::DifferentInputs);
            };
            res.pop(expected_type);
        }

        for (i, t) in left.pushes.iter().enumerate() {
            match t.union(&right.pushes[i]) {
                Ok(t) => res.push(t),
                Err(()) => {
                    todo!("Handle union error");
                }
            }
        }

        Ok(res)
    }

    pub fn with_serial(mut self, other: &Arity) -> Arity {
        self.serial(other);
        self
    }

    // pops1 + pushes1 + pops2 + pushes2
    // (int int -- int) + (int int -- int) = (int int int -- int)

    // (a -- b) + (b -- ) => (a -- )
    // (a -- b) + (b -- ) => (a -- )
    // (a b -- c d) + (c d -- e f) = (a b -- e f)
    // (a b -- c d) + (e c d -- f) = (e a b -- f)
}

impl std::fmt::Debug for Arity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", &self.stringify())
    }
}

#[cfg(test)]
mod tests {
    use crate::lang::{Arity, Type};

    #[test]
    fn two_pushes() {
        assert_eq!(
            Arity::literal(Type::Number).with_serial(&Arity::literal(Type::Number)),
            Arity::noop()
                .with_push(Type::Number)
                .with_push(Type::Number),
        );
    }

    #[test]
    fn push_pop() {
        assert_eq!(
            Arity::literal(Type::Number).with_serial(&Arity::noop().with_pop(Type::Number)),
            Arity::noop(),
        );
    }

    #[test]
    fn combine_number_binary() {
        assert_eq!(
            Arity::number_binary().with_serial(&Arity::number_binary()),
            Arity::noop()
                .with_pop(Type::Number)
                .with_pop(Type::Number)
                .with_pop(Type::Number)
                .with_push(Type::Number)
        );
    }

    #[test]
    fn combine_number_unary() {
        assert_eq!(
            Arity::number_binary().with_serial(&Arity::number_unary()),
            Arity::noop()
                .with_pop(Type::Number)
                .with_pop(Type::Number)
                .with_push(Type::Number)
        );
    }

    #[test]
    fn number_reachover() {
        assert_eq!(
            Arity::literal(Type::Number).with_serial(&Arity::number_binary()),
            Arity::noop().with_pop(Type::Number).with_push(Type::Number)
        );
    }
}
