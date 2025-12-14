#[cfg(test)]
mod tests {
    use crate::lang::{Block, Branch, Function, Loop, Module, Term};
    use crate::parser::parse;

    #[test]
    fn basic_add() {
        let ast = Module {
            body: Block {
                terms: vec![420f64.into(), 42f64.into(), Term::Name("+".into())],
            },
            ..Default::default()
        };

        let code = r"420 42 +";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn basic_subtract() {
        let ast = Module {
            body: Block {
                terms: vec![420f64.into(), 42f64.into(), Term::Name("-".into())],
            },
            ..Default::default()
        };

        let code = r"420 42 -";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn basic_multiply() {
        let ast = Module {
            body: Block {
                terms: vec![20f64.into(), 4f64.into(), Term::Name("*".into())],
            },
            ..Default::default()
        };

        let code = r"20 4 *";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn basic_divide() {
        let ast = Module {
            body: Block {
                terms: vec![20f64.into(), 4f64.into(), Term::Name("/".into())],
            },
            ..Default::default()
        };

        let code = r"20 4 /";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn basic_or() {
        let ast = Module {
            body: Block {
                terms: vec![true.into(), false.into(), Term::Name("||".into())],
            },
            ..Default::default()
        };

        let code = r"true false ||";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn basic_or2() {
        let ast = Module {
            body: Block {
                terms: vec![false.into(), false.into(), Term::Name("||".into())],
            },
            ..Default::default()
        };

        let code = r"false false ||";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn basic_or3() {
        let ast = Module {
            body: Block {
                terms: vec![true.into(), true.into(), Term::Name("||".into())],
            },
            ..Default::default()
        };

        let code = r"true true ||";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn basic_and() {
        let ast = Module {
            body: Block {
                terms: vec![true.into(), false.into(), Term::Name("&&".into())],
            },
            ..Default::default()
        };

        let code = r"true false &&";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn basic_and2() {
        let ast = Module {
            body: Block {
                terms: vec![false.into(), false.into(), Term::Name("&&".into())],
            },
            ..Default::default()
        };

        let code = r"false false &&";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn basic_and3() {
        let ast = Module {
            body: Block {
                terms: vec![true.into(), true.into(), Term::Name("&&".into())],
            },
            ..Default::default()
        };

        let code = r"true true &&";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
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
            ..Default::default()
        };

        let code = r"generate: {36 6 +} generate";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
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
            ..Default::default()
        };

        let code = r"rfib: {{(dup 1 >) 1 - dup rfib swap 1 - rfib + }} 5 rfib";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
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
            ..Default::default()
        };

        let code = r"ifib: {0 1 [(rot dup) 1 - rot rot dup rot +] drop drop} 20 ifib";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn comments() {
        let code = r#"
        1
        // comment
2 // comment 2
3
        "#;

        let ast = Module {
            body: Block {
                terms: vec![1.into(), 2.into(), 3.into()],
            },
            ..Default::default()
        };
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }
}
