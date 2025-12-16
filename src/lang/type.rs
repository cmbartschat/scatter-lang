#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Type {
    Bool,
    Number,
    String,
    Address,
    Unknown,
}

impl Type {
    pub fn assignable_to(&self, other: &Self) -> bool {
        if other == &Self::Unknown {
            return true;
        }
        if self == other {
            return true;
        }
        false
    }

    pub fn stringify(&self) -> &'static str {
        match self {
            Type::Bool => "b",
            Type::Number => "n",
            Type::String => "s",
            Type::Unknown => "u",
            Type::Address => "a",
        }
    }

    pub fn union(self, other: Self) -> Self {
        if self.assignable_to(&other) {
            other
        } else if other.assignable_to(&self) {
            self
        } else {
            Self::Unknown
        }
    }

    pub fn inter(self, other: Self) -> Option<Self> {
        if self == other {
            Some(self)
        } else if self == Self::Unknown {
            Some(other)
        } else if other == Self::Unknown {
            Some(self)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lang::Type;

    #[test]
    fn assignable_to() {
        assert!(Type::Bool.assignable_to(&Type::Bool));
        assert!(!Type::Bool.assignable_to(&Type::Number));
        assert!(!Type::Bool.assignable_to(&Type::String));
        assert!(Type::Bool.assignable_to(&Type::Unknown));

        assert!(!Type::Unknown.assignable_to(&Type::Bool));
        assert!(!Type::Unknown.assignable_to(&Type::Number));
        assert!(!Type::Unknown.assignable_to(&Type::String));
        assert!(Type::Unknown.assignable_to(&Type::Unknown));
    }

    #[test]
    fn union() {
        assert_eq!(Type::Bool.union(Type::Bool), Type::Bool);
        assert_eq!(Type::Number.union(Type::Number), Type::Number);
        assert_eq!(Type::String.union(Type::String), Type::String);
        assert_eq!(Type::Unknown.union(Type::Unknown), Type::Unknown);

        assert_eq!(Type::Bool.union(Type::Unknown), Type::Unknown);
        assert_eq!(Type::Bool.union(Type::Number), Type::Unknown);
        assert_eq!(Type::Unknown.union(Type::String), Type::Unknown);
    }

    #[test]
    fn inter() {
        assert_eq!(Type::Bool.inter(Type::Bool), Some(Type::Bool));
        assert_eq!(Type::Number.inter(Type::Number), Some(Type::Number));
        assert_eq!(Type::String.inter(Type::String), Some(Type::String));
        assert_eq!(Type::Unknown.inter(Type::Unknown), Some(Type::Unknown));

        assert_eq!(Type::Bool.inter(Type::Unknown), Some(Type::Bool));
        assert_eq!(Type::Bool.inter(Type::Number), None);
        assert_eq!(Type::Unknown.inter(Type::String), Some(Type::String));
    }
}
