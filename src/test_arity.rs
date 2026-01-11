#[cfg(test)]
mod tests {
    use crate::lang::{Arity, ArityCombineError, Type};

    fn check_serial(first: &str, second: &str, expected: &str) {
        assert_eq!(
            Arity::serial(
                &Arity::parse(first).unwrap(),
                &Arity::parse(second).unwrap(),
            )
            .unwrap()
            .stringify(),
            expected,
        );
    }

    fn check_serial_err(first: &str, second: &str, expected: &ArityCombineError) {
        assert_eq!(
            &Arity::serial(
                &Arity::parse(first).unwrap(),
                &Arity::parse(second).unwrap(),
            )
            .unwrap_err(),
            expected,
        );
    }

    fn check_parallel(first: &str, second: &str, expected: &str) {
        assert_eq!(
            Arity::parallel(
                &Arity::parse(first).unwrap(),
                &Arity::parse(second).unwrap(),
            )
            .unwrap()
            .stringify(),
            expected,
        );
    }

    fn check_parallel_err(first: &str, second: &str, expected: &ArityCombineError) {
        assert_eq!(
            &Arity::parallel(
                &Arity::parse(first).unwrap(),
                &Arity::parse(second).unwrap(),
            )
            .unwrap_err(),
            expected,
        );
    }

    #[test]
    fn parse() {
        assert_eq!(
            Arity::parse("n b s a u -").unwrap(),
            (
                vec![
                    Type::Unknown,
                    Type::Address,
                    Type::String,
                    Type::Bool,
                    Type::Number,
                ],
                vec![]
            )
                .into()
        );

        assert_eq!(
            Arity::parse("n b s a u -").unwrap().stringify(),
            "n b s a u -"
        );

        assert_eq!(
            Arity::parse("u u - 0|1").unwrap(),
            Arity::generic_1(2, (0, 1))
        );

        {
            let source = "-";
            assert_eq!(Arity::parse(source).unwrap().stringify(), source);
        }

        {
            let source = "0 - 0 0";
            assert_eq!(Arity::parse(source).unwrap().stringify(), source);
        }

        {
            let source = "0 - 0 0, a:n";
            assert_eq!(Arity::parse(source).unwrap().stringify(), source);
        }

        {
            let source = "0 -, a:0";
            assert_eq!(Arity::parse(source).unwrap().stringify(), source);
        }

        {
            let source = "- u, >a";
            assert_eq!(Arity::parse(source).unwrap().stringify(), source);
        }

        {
            let source = "0 -, a:0?";
            assert_eq!(Arity::parse(source).unwrap().stringify(), source);
        }

        {
            let source = "- u, a:u?";
            assert_eq!(Arity::parse(source).unwrap().stringify(), source);
        }
    }

    #[test]
    fn serial() {
        check_serial("- n", "- n", "- n n");
        check_serial("- n", "n -", "-");
        check_serial("n n - n", "n n - n", "n n n - n");
        check_serial("n n - n", "n - n", "n n - n");
        check_serial("- n", "n n - n", "n - n");
        check_serial("n - s", "s - n", "n - n");
        check_serial("n - u", "u - n", "n - n");
        check_serial("u - 0", "n - s", "n - s");
        check_serial("u u - 0|1", "n - s", "n n - s");
        check_serial("u u - 0|1", "u - u", "u u - u");
        check_serial("- n s", "u u - 0|1", "- u");
    }

    #[test]
    fn serial_error_1() {
        check_serial_err("n - s", "n - s", &ArityCombineError::IncompatibleTypes);
    }

    #[test]
    fn serial_error_2() {
        check_serial_err("n - b s", "n s - s", &ArityCombineError::IncompatibleTypes);
    }

    #[test]
    fn serial_error_3() {
        check_serial_err(
            "u u - 0|1 0|1",
            "n s - s",
            &ArityCombineError::IncompatibleTypes,
        );
    }

    #[test]
    fn parallel() {
        check_parallel("- n", "- s", "- u");
    }

    #[test]
    fn parallel_error_1() {
        check_parallel_err("n -", "s -", &ArityCombineError::IncompatibleTypes);
    }

    #[test]
    fn parallel_2() {
        check_parallel("n - s", "-", "n - u");
    }

    #[test]
    fn parallel_3() {
        check_parallel("u u - 0|1", "n s - u", "n s - u");
    }

    #[test]
    fn parallel_4() {
        check_parallel("-", "n s - n n", "n s - n u");
    }

    #[test]
    fn parallel_5() {
        check_parallel("u - 0", "n s - n n", "n s - n u");
    }

    #[test]
    fn parallel_6() {
        check_parallel("u u - 1 0", "n s - n n", "n s - n u");
    }

    #[test]
    fn parallel_7() {
        check_parallel("u u - 0 1", "n s - n n", "n s - u n");
    }
}
