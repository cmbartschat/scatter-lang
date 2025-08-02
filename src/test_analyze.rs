#[cfg(test)]
mod tests {
    use crate::analyze::{AnalysisError, analyze};

    use crate::parser::parse;

    fn assert_fn_arity(code: &str, expected_arity: &str) {
        let ast = parse(code).unwrap();
        let a = analyze(&ast);
        let actual_arity = a.arities.get("fn").unwrap().as_ref().unwrap().stringify();
        assert_eq!(actual_arity, expected_arity);
    }

    fn assert_fn_unset(code: &str) {
        let ast = parse(code).unwrap();
        let a = analyze(&ast);
        assert_eq!(a.arities.get("fn"), None);
    }

    fn assert_fn_err(code: &str, expected_error: AnalysisError) {
        let ast = parse(code).unwrap();
        let a = analyze(&ast);
        assert_eq!(a.arities.get("fn").unwrap(), &Err(expected_error));
    }

    #[test]
    fn basic_numbers() {
        assert_fn_arity("fn: {1 1}", "- n n");
    }

    #[test]
    fn basic_add() {
        assert_fn_arity("fn: {+}", "n n - n");
    }

    #[test]
    fn nested_function() {
        assert_fn_arity("fn2: {+} fn: {fn2 fn2}", "n n n - n");
    }

    #[test]
    fn recursive_function() {
        assert_fn_unset("fn: {fn}");
    }

    #[test]
    fn mutually_recursive_function() {
        assert_fn_unset("fn: {fn2} fn2: {fn}");
    }

    #[test]
    fn complex_function1() {
        assert_fn_arity("fn: {1 + !}", "n - b");
    }

    #[test]
    fn complex_function2() {
        assert_fn_arity("fn: {1 + ! 3 &&}", "n - u");
    }

    #[test]
    fn literals() {
        assert_fn_arity("fn: {1 true \"test\"}", "- n b s");
    }

    #[test]
    fn branch_1() {
        assert_fn_arity("fn: {{(0) undefined (1) 3}}", "- n");
    }

    #[test]
    fn branch_2() {
        assert_fn_arity(
            "fn: {{(0) undefined (false) null (1) 3 (false) true}}",
            "- n",
        );
    }

    #[test]
    fn branch_3() {
        assert_fn_arity("fn: {{(0 !) 1 (0 !) 2 (1) 3}}", "- n");
    }

    #[test]
    fn branch_4() {
        assert_fn_arity("fn: {{(1) true (dup) - - - - () 1 1 1 1}}", "- b");
    }

    #[test]
    fn loop_1() {
        assert_fn_arity("fn: {[(dup) 1 -]}", "n - n");
    }

    #[test]
    fn loop_2() {
        assert_fn_arity("fn: {[(dup) swap]}", "1 0 - 0|1 0|1");
    }

    #[test]
    fn loop_3() {
        assert_fn_arity("fn: {[(dup) swap]}", "1 0 - 0|1 0|1");
    }

    #[test]
    fn loop_4() {
        assert_fn_err("fn: {[(dup) 1]}", AnalysisError::IndefiniteSize);
    }

    #[test]
    fn test_intrinsic_or() {
        assert_fn_arity("fn: {||}", "1 0 - 0|1");
    }

    #[test]
    fn test_intrinsic_and() {
        assert_fn_arity("fn: {&&}", "1 0 - 0|1");
    }

    #[test]
    fn test_intrinsic_swap() {
        assert_fn_arity("fn: {swap}", "1 0 - 0 1");
    }

    #[test]
    fn test_intrinsic_dup() {
        assert_fn_arity("fn: {dup}", "0 - 0 0");
    }

    #[test]
    fn test_intrinsic_over() {
        assert_fn_arity("fn: {over}", "1 0 - 1 0 1");
    }

    #[test]
    fn test_intrinsic_rot() {
        assert_fn_arity("fn: {rot}", "2 1 0 - 1 0 2");
    }

    #[test]
    fn test_intrinsic_drop() {
        assert_fn_arity("fn: {drop}", "u -");
    }

    #[test]
    fn test_generic_1() {
        assert_fn_arity("fn: {swap swap}", "1 0 - 1 0");
    }

    #[test]
    fn test_generic_2() {
        assert_fn_arity("fn: {swap +}", "n n - n");
    }

    #[test]
    fn test_generic_3() {
        assert_fn_arity("fn: {swap ++}", "n 0 - 0 n");
    }

    #[test]
    fn test_generic_4() {
        assert_fn_arity("fn: {dup ++}", "n - n n");
    }

    #[test]
    fn test_generic_5() {
        assert_fn_arity("fn: {1 dup}", "- n n");
    }
}
