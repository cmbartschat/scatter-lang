#[cfg(test)]
mod tests {
    use crate::intrinsics::get_intrinsics;
    use crate::lang::Value;
    use crate::{interpreter::Interpreter, parser::parse};

    fn interpret_str(source: &str) -> Vec<Value> {
        let ast = parse(source).unwrap();
        let mut ctx = Interpreter::new();
        ctx.load(&ast).expect("Execution error");
        ctx.stack
    }

    static E2E_TESTS: &str = include_str!("../examples/e2e.sl");

    #[test]
    fn e2e() {
        assert_eq!(interpret_str(E2E_TESTS), vec![]);
    }

    #[test]
    fn exhaustive() {
        for i in get_intrinsics() {
            if i.0 == "assert" || i.0 == "print" {
                continue;
            }
            let pattern = format!("\"{}\" start_suite", i.0);
            assert!(
                E2E_TESTS.contains(&pattern),
                "Should include testing for {}",
                i.0
            );
        }
    }
}
