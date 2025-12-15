#[cfg(test)]
mod tests {
    use crate::intrinsics::{get_c_name, get_intrinsics};
    use crate::lang::ImportNaming;
    use crate::program::{NamespaceImport, Program};
    use crate::{interpreter::Interpreter, parser::parse};

    static TEST_HELPERS: &str = include_str!("../examples/test.sl");
    static E2E_TESTS: &str = include_str!("../examples/e2e.sl");

    #[test]
    fn e2e() {
        let helpers_ast = parse(TEST_HELPERS).unwrap();
        let ast = parse(E2E_TESTS).unwrap();
        let mut program = Program::new_from_module(&ast);
        let helpers_namespace = program.allocate_namespace();
        program.add_functions(helpers_namespace, &helpers_ast.functions);
        program.add_imports(
            0,
            vec![NamespaceImport {
                id: helpers_namespace,
                naming: ImportNaming::Wildcard,
            }],
        );

        let ctx = Interpreter::begin(&program);
        assert_eq!(ctx.execute(&ast.body).unwrap().stack, vec![]);
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

    #[test]
    fn intrinsics_codegen_test() {
        static C_DEFINITIONS: &'static str = include_str!("./codegen/c.h");
        static JS_DEFINITIONS: &'static str = include_str!("./codegen/js.js");

        let js_exceptions = ["readline"];

        for name in get_intrinsics().iter().map(|f| get_c_name(f.0)) {
            assert!(
                C_DEFINITIONS.contains(&format!("status_t {name}(void) {{")),
                "defined by c: {name}",
            );
            if !js_exceptions.contains(&name) {
                assert!(
                    JS_DEFINITIONS.contains(&format!("function {name}() {{")),
                    "defined by js: {name}",
                );
            }
        }
    }
}
