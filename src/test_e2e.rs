#[cfg(test)]
mod tests {
    use crate::codegen;
    use crate::intrinsics::{IntrinsicData, get_intrinsic_codegen_name, get_intrinsics};
    use crate::lang::{ImportNaming, Module};
    use crate::program::{FunctionOverwriteStrategy, NamespaceImport, Program};
    use crate::{interpreter::Interpreter, parser::parse};

    static TEST_HELPERS: &str = include_str!("../examples/test.sl");
    static E2E_TESTS: &str = include_str!("../examples/e2e.sl");

    fn get_e2e_program() -> (Module, Program) {
        let helpers_ast = parse(TEST_HELPERS).unwrap();
        let ast = parse(E2E_TESTS).unwrap();
        let mut program = Program::new_from_module(&ast);
        let helpers_namespace = program.allocate_namespace();
        program
            .add_functions(
                helpers_namespace,
                &helpers_ast.functions,
                FunctionOverwriteStrategy::FailOnDuplicate,
            )
            .unwrap();
        program.add_imports(
            0,
            vec![NamespaceImport {
                id: helpers_namespace,
                naming: ImportNaming::Wildcard,
            }],
        );
        (ast, program)
    }

    #[test]
    fn e2e() {
        let (ast, program) = get_e2e_program();
        let ctx = Interpreter::begin(&program);
        assert_eq!(ctx.execute(0, &ast.body).unwrap().stack, vec![]);
    }

    static SKIPPED_INTRINSICS: [&str; 3] = ["assert", "print", "readline"];

    #[test]
    fn exhaustive() {
        for IntrinsicData { name, .. } in get_intrinsics() {
            if SKIPPED_INTRINSICS.contains(name) {
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
        for IntrinsicData { name, .. } in get_intrinsics() {
            let c_name = get_intrinsic_codegen_name(name).unwrap();
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
    fn codegen_rs_test() {
        let (ast, program) = get_e2e_program();
        assert!(
            500 < codegen::rs::rs_codegen_module(&program, 0, &ast.body)
                .unwrap()
                .len()
        );
    }

    #[test]
    fn codegen_js_test() {
        let (ast, program) = get_e2e_program();
        assert!(
            500 < codegen::js::js_codegen_module(&program, 0, &ast.body)
                .unwrap()
                .len()
        );
    }

    #[test]
    fn codegen_c_test() {
        let (ast, program) = get_e2e_program();
        assert!(
            500 < codegen::c::c_codegen_module(&program, 0, &ast.body)
                .unwrap()
                .len()
        );
    }

    #[test]
    fn intrinsics_codegen_test() {
        static C_DEFINITIONS: &str = include_str!("./codegen/c.h");
        static JS_DEFINITIONS: &str = include_str!("./codegen/js.js");

        let js_exceptions = [];
        let c_exceptions = [];

        for name in get_intrinsics()
            .iter()
            .map(|f| get_intrinsic_codegen_name(f.name).unwrap())
        {
            let c_should_have = !c_exceptions.contains(&name);
            assert_eq!(
                c_should_have,
                C_DEFINITIONS.contains(&format!("status_t {name}(void) {{")),
                "defined by c: {name}",
            );

            let js_should_have = !js_exceptions.contains(&name);
            assert_eq!(
                js_should_have,
                JS_DEFINITIONS.contains(&format!("function {name}() {{")),
                "defined by js: {name}",
            );
        }
    }
}
