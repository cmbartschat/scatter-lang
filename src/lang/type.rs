#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Type {
    Bool,
    Number,
    String,
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
}
