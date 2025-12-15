#[cfg(test)]
mod tests {
    use crate::intrinsics::{get_c_name, get_intrinsics};
    use crate::{interpreter::Interpreter, parser::parse};

    static TEST_HELPERS: &str = include_str!("../examples/test.sl");
    static E2E_TESTS: &str = include_str!("../examples/e2e.sl");

    #[test]
    fn e2e() {
        let helpers_ast = parse(TEST_HELPERS).unwrap();
        let ast = parse(E2E_TESTS).unwrap();
        let mut ctx = Interpreter::new();
        ctx.load_functions(&helpers_ast).unwrap();
        ctx.load_functions(&ast).unwrap();
        ctx.evaluate_block(&ast.body).unwrap();
        assert_eq!(ctx.stack, vec![]);
    }

    static SKIPPED_INTRINSICS: [&str; 3] = ["assert", "print", "readline"];

    #[test]
    fn exhaustive() {
        for (name, _) in get_intrinsics().iter() {
            if SKIPPED_INTRINSICS.contains(&name.as_str()) {
                continue;
            }
            let pattern = format!("\"{}\" start_suite", name);
            assert!(
                E2E_TESTS.contains(&pattern),
                "Should include testing for {}",
                name
            );
        }
    }

    #[test]
    fn intrinsics_symbols() {
        for (name, _) in get_intrinsics().iter() {
            let c_name = get_c_name(name);
            for c in c_name.chars() {
                assert!(
                    matches!(c, '0'..='9' | 'a'..='z' | '_'),
                    "c_name: {} should not include special characters",
                    c_name
                );
            }
        }
    }
}
