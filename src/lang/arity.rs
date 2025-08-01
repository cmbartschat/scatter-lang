use crate::lang::Type;

#[derive(Clone, PartialEq, Debug)]
pub struct Arity {
    pub pops: Vec<Type>,
    pub pushes: Vec<Type>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ArityCombineError {
    DifferingSizes,
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

    pub fn number_binary() -> Self {
        Self::binary(Type::Number, Type::Number, Type::Number)
    }

    pub fn any_binary() -> Self {
        Self::binary(Type::Unknown, Type::Unknown, Type::Unknown)
    }

    pub fn number_unary() -> Self {
        Self::unary(Type::Number, Type::Number)
    }

    pub fn pop(&mut self, term: Type) {
        if self.pushes.pop().is_none() {
            self.pops.push(term);
        };
    }

    pub fn push(&mut self, term: Type) {
        self.pushes.push(term);
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
        other.pops.iter().for_each(|f| self.pop(f.to_owned()));
        other.pushes.iter().for_each(|f| self.push(f.to_owned()));
    }

    pub fn stringify(&self) -> String {
        let mut res = String::new();
        for pop in self.pops.iter().rev() {
            res.push_str(pop.stringify());
            res.push(' ');
        }

        res.push('-');

        for push in self.pushes.iter() {
            res.push(' ');
            res.push_str(push.stringify());
        }

        res
    }

    pub fn parallel(left: &Arity, other: &Arity) -> Result<Arity, ArityCombineError> {
        if left.size() != other.size() {
            return Err(ArityCombineError::DifferingSizes);
        }
        let mut res = Arity::noop();

        for (i, t) in left.pops.iter().enumerate() {
            let other_type = other.pops[i];
            if t.assignable_to(&other_type) {
                res.pop(other_type);
            } else if other_type.assignable_to(t) {
                res.pop(*t);
            } else {
                res.pop(Type::Unknown);
            }
        }

        for (i, t) in left.pushes.iter().enumerate() {
            let other_type = other.pushes[i];
            if t.assignable_to(&other_type) {
                res.push(*t);
            } else if other_type.assignable_to(t) {
                res.push(other_type);
            } else {
                res.push(Type::Unknown);
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
