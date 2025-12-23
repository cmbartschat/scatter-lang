#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use crate::lang::{
        Block, Branch, Function, ImportNaming, Loop, Module, OwnedValue, SourceLocation,
        SourceRange, Term,
    };
    use crate::parser::parse;
    use crate::program::{NamespaceImport, Program};

    fn interpret(ast: &Module) -> Vec<OwnedValue> {
        Interpreter::begin(&Program::new_from_module(ast))
            .execute(&ast.body)
            .unwrap()
            .stack
    }

    fn name<T: Into<String>>(t: T) -> Term {
        Term::Name(
            t.into(),
            SourceRange {
                start: SourceLocation::start(),
                end: SourceLocation::start(),
            },
        )
    }

    #[test]
    fn basic_add() {
        let ast = Module {
            body: Block {
                terms: vec![420f64.into(), 42f64.into(), name("+")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![462f64.into()]);
    }

    #[test]
    fn basic_subtract() {
        let ast = Module {
            body: Block {
                terms: vec![420f64.into(), 42f64.into(), name("-")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![378f64.into()]);
    }

    #[test]
    fn basic_multiply() {
        let ast = Module {
            body: Block {
                terms: vec![20f64.into(), 4f64.into(), name("*")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![80f64.into()]);
    }

    #[test]
    fn basic_divide() {
        let ast = Module {
            body: Block {
                terms: vec![20f64.into(), 4f64.into(), name("/")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![5f64.into()]);
    }

    #[test]
    fn basic_or() {
        let ast = Module {
            body: Block {
                terms: vec![true.into(), false.into(), name("||")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![true.into()]);
    }

    #[test]
    fn basic_or2() {
        let ast = Module {
            body: Block {
                terms: vec![false.into(), false.into(), name("||")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![false.into()]);
    }

    #[test]
    fn basic_or3() {
        let ast = Module {
            body: Block {
                terms: vec![true.into(), true.into(), name("||")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![true.into()]);
    }

    #[test]
    fn basic_and() {
        let ast = Module {
            body: Block {
                terms: vec![true.into(), false.into(), name("&&")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![false.into()]);
    }

    #[test]
    fn basic_and2() {
        let ast = Module {
            body: Block {
                terms: vec![false.into(), false.into(), name("&&")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![false.into()]);
    }

    #[test]
    fn basic_and3() {
        let ast = Module {
            body: Block {
                terms: vec![true.into(), true.into(), name("&&")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![true.into()]);
    }

    #[test]
    fn function_call() {
        let ast = Module {
            functions: vec![Function {
                name: "generate".into(),
                body: Block {
                    terms: vec![36.into(), 6.into(), name("+")],
                },
            }],
            body: Block {
                terms: vec![name("generate")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![42.into()]);
    }

    #[test]
    fn fib_recursive() {
        let ast = Module {
            functions: vec![Function {
                name: "rfib".into(),
                body: Block {
                    terms: vec![Term::Branch(Branch {
                        arms: vec![(
                            Block {
                                terms: vec![name("dup"), 1.into(), name(">")],
                            },
                            Block {
                                terms: vec![
                                    1.into(),
                                    name("-"),
                                    name("dup"),
                                    name("rfib"),
                                    name("swap"),
                                    1.into(),
                                    name("-"),
                                    name("rfib"),
                                    name("+"),
                                ],
                            },
                        )],
                    })],
                },
            }],
            body: Block {
                terms: vec![5.into(), name("rfib")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![5.into()]);
    }

    #[test]
    fn fib_iterative() {
        let ast = Module {
            functions: vec![Function {
                name: "ifib".into(),
                body: Block {
                    terms: vec![
                        0.into(),
                        1.into(),
                        Term::Loop(Loop {
                            pre_condition: Some(Block {
                                terms: vec![name("rot"), name("dup")],
                            }),
                            post_condition: None,
                            body: Block {
                                terms: vec![
                                    1.into(),
                                    name("-"),
                                    name("rot"),
                                    name("rot"),
                                    name("dup"),
                                    name("rot"),
                                    name("+"),
                                ],
                            },
                        }),
                        name("drop"),
                        name("drop"),
                    ],
                },
            }],
            body: Block {
                terms: vec![20.into(), name("ifib")],
            },
            ..Default::default()
        };

        let result = interpret(&ast);

        assert_eq!(result, vec![6765.into()]);
    }

    #[test]
    fn imports() {
        let main = parse("helper1 helper2 helper3.helper3").unwrap();
        let helper1 = parse("helper1: 1").unwrap();
        let helper2 = parse("helper2: 2").unwrap();
        let helper3 = parse("helper3: 3").unwrap();

        let mut program = Program::new();

        let main_id = program.allocate_namespace();

        let helper1_id = program.allocate_namespace();
        program.add_functions(helper1_id, &helper1.functions);

        let helper2_id = program.allocate_namespace();
        program.add_functions(helper2_id, &helper2.functions);

        let helper3_id = program.allocate_namespace();
        program.add_functions(helper3_id, &helper3.functions);

        program.add_imports(
            main_id,
            vec![
                NamespaceImport {
                    id: helper1_id,
                    naming: ImportNaming::Wildcard,
                },
                NamespaceImport {
                    id: helper2_id,
                    naming: ImportNaming::Named(vec!["helper2".into()]),
                },
                NamespaceImport {
                    id: helper3_id,
                    naming: ImportNaming::Scoped("helper3".into()),
                },
            ],
        );

        let interpreter = Interpreter::begin(&program);
        let result = interpreter.execute(&main.body).unwrap().stack;
        assert_eq!(result, vec![1.into(), 2.into(), 3.into()]);
    }
}
