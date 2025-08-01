#[cfg(test)]
mod tests {
    use crate::interpreter::{Interpreter, Stack};
    use crate::lang::{Block, Branch, Function, Loop, Module, Term};

    fn interpret(program: Module) -> Stack {
        let mut ctx = Interpreter::new();
        ctx.load(&program).expect("Execution error");
        ctx.stack
    }

    #[test]
    fn basic_add() {
        let ast = Module {
            functions: vec![],
            body: Block {
                terms: vec![420f64.into(), 42f64.into(), Term::Name("+".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![462f64.into()])
    }

    #[test]
    fn basic_subtract() {
        let ast = Module {
            functions: vec![],
            body: Block {
                terms: vec![420f64.into(), 42f64.into(), Term::Name("-".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![378f64.into()])
    }

    #[test]
    fn basic_multiply() {
        let ast = Module {
            functions: vec![],
            body: Block {
                terms: vec![20f64.into(), 4f64.into(), Term::Name("*".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![80f64.into()])
    }

    #[test]
    fn basic_divide() {
        let ast = Module {
            functions: vec![],
            body: Block {
                terms: vec![20f64.into(), 4f64.into(), Term::Name("/".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![5f64.into()])
    }

    #[test]
    fn basic_or() {
        let ast = Module {
            functions: vec![],
            body: Block {
                terms: vec![true.into(), false.into(), Term::Name("||".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![true.into()])
    }

    #[test]
    fn basic_or2() {
        let ast = Module {
            functions: vec![],
            body: Block {
                terms: vec![false.into(), false.into(), Term::Name("||".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![false.into()])
    }

    #[test]
    fn basic_or3() {
        let ast = Module {
            functions: vec![],
            body: Block {
                terms: vec![true.into(), true.into(), Term::Name("||".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![true.into()])
    }

    #[test]
    fn basic_and() {
        let ast = Module {
            functions: vec![],
            body: Block {
                terms: vec![true.into(), false.into(), Term::Name("&&".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![false.into()])
    }

    #[test]
    fn basic_and2() {
        let ast = Module {
            functions: vec![],
            body: Block {
                terms: vec![false.into(), false.into(), Term::Name("&&".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![false.into()])
    }

    #[test]
    fn basic_and3() {
        let ast = Module {
            functions: vec![],
            body: Block {
                terms: vec![true.into(), true.into(), Term::Name("&&".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![true.into()])
    }

    #[test]
    fn function_call() {
        let ast = Module {
            functions: vec![Function {
                name: "generate".into(),
                body: Block {
                    terms: vec![36.into(), 6.into(), Term::Name("+".into())],
                },
            }],
            body: Block {
                terms: vec![Term::Name("generate".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![42.into()])
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
                                terms: vec![
                                    Term::Name("dup".into()),
                                    1.into(),
                                    Term::Name(">".into()),
                                ],
                            },
                            Block {
                                terms: vec![
                                    1.into(),
                                    Term::Name("-".into()),
                                    Term::Name("dup".into()),
                                    Term::Name("rfib".into()),
                                    Term::Name("swap".into()),
                                    1.into(),
                                    Term::Name("-".into()),
                                    Term::Name("rfib".into()),
                                    Term::Name("+".into()),
                                ],
                            },
                        )],
                    })],
                },
            }],
            body: Block {
                terms: vec![5.into(), Term::Name("rfib".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![5.into()])
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
                                terms: vec![Term::Name("rot".into()), Term::Name("dup".into())],
                            }),
                            post_condition: None,
                            body: Block {
                                terms: vec![
                                    1.into(),
                                    Term::Name("-".into()),
                                    Term::Name("rot".into()),
                                    Term::Name("rot".into()),
                                    Term::Name("dup".into()),
                                    Term::Name("rot".into()),
                                    Term::Name("+".into()),
                                ],
                            },
                        }),
                        Term::Name("drop".into()),
                        Term::Name("drop".into()),
                    ],
                },
            }],
            body: Block {
                terms: vec![20.into(), Term::Name("ifib".into())],
            },
        };

        let result = interpret(ast);

        assert_eq!(result, vec![6765.into()])
    }
}
