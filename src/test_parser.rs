#[cfg(test)]
mod tests {
    use crate::lang::{
        Block, Branch, Function, Import, ImportLocation, ImportNaming, Loop, Module,
        SourceLocation, SourceRange, Symbol, Term,
    };
    use crate::parse_error::{ParseError, UnexpectedError};
    use crate::parser::parse;

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

        let code = r"420 42 +";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn basic_subtract() {
        let ast = Module {
            body: Block {
                terms: vec![420f64.into(), 42f64.into(), name("-")],
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
                terms: vec![20f64.into(), 4f64.into(), name("*")],
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
                terms: vec![20f64.into(), 4f64.into(), name("/")],
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
                terms: vec![true.into(), false.into(), name("||")],
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
                terms: vec![false.into(), false.into(), name("||")],
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
                terms: vec![true.into(), true.into(), name("||")],
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
                terms: vec![true.into(), false.into(), name("&&")],
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
                terms: vec![false.into(), false.into(), name("&&")],
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
                terms: vec![true.into(), true.into(), name("&&")],
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
                    terms: vec![36.into(), 6.into(), name("+")],
                },
            }],
            body: Block {
                terms: vec![name("generate")],
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

        let code = r"ifib: {0 1 [(rot dup) 1 - rot rot dup rot +] drop drop} 20 ifib";
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn comments_1() {
        let code = r"
        1
        // comment
2 // comment 2
3
        ";

        let ast = Module {
            body: Block {
                terms: vec![1.into(), 2.into(), 3.into()],
            },
            ..Default::default()
        };
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn comments_2() {
        let code = "1//1\n2//2\n3//3";

        let ast = Module {
            body: Block {
                terms: vec![1.into(), 2.into(), 3.into()],
            },
            ..Default::default()
        };
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn imports_1() {
        let code = r#"
# {name1 name2} "./1.sl" 
# scope "./2.sl"
# * "./3.sl"
        "#;

        let ast = Module {
            imports: vec![
                Import {
                    naming: ImportNaming::Named(vec!["name1".into(), "name2".into()]),
                    location: ImportLocation::Relative("./1.sl".into()),
                },
                Import {
                    naming: ImportNaming::Scoped("scope".into()),
                    location: ImportLocation::Relative("./2.sl".into()),
                },
                Import {
                    naming: ImportNaming::Wildcard,
                    location: ImportLocation::Relative("./3.sl".into()),
                },
            ],
            ..Default::default()
        };
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn imports_2() {
        let code = r#"
# {name1 name2} "./1.sl" 
        "#;

        let ast = Module {
            imports: vec![Import {
                naming: ImportNaming::Named(vec!["name1".into(), "name2".into()]),
                location: ImportLocation::Relative("./1.sl".into()),
            }],
            ..Default::default()
        };
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn imports_3() {
        let code = r#"
# scope "./2.sl"
        "#;

        let ast = Module {
            imports: vec![Import {
                naming: ImportNaming::Scoped("scope".into()),
                location: ImportLocation::Relative("./2.sl".into()),
            }],
            ..Default::default()
        };
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn imports_4() {
        let code = r#"
# * "./3.sl"
        "#;

        let ast = Module {
            imports: vec![Import {
                naming: ImportNaming::Wildcard,
                location: ImportLocation::Relative("./3.sl".into()),
            }],
            ..Default::default()
        };
        let result = parse(code).unwrap();
        assert_eq!(result, ast);
    }

    #[test]
    fn single_character() {
        let code = "4";
        let result = parse(code).unwrap();
        let ast = Module {
            body: Block {
                terms: vec![Term::Number(4.)],
            },
            ..Default::default()
        };
        assert_eq!(result, ast);
    }

    #[test]
    fn multiline_1() {
        let code = r"
/* 1 */
";
        let result = parse(code).unwrap();
        let ast = Module {
            body: Block { terms: vec![] },
            ..Default::default()
        };
        assert_eq!(result, ast);
    }

    #[test]
    fn multiline_2() {
        let code = r"
/* 1 / */
";
        let result = parse(code).unwrap();
        let ast = Module {
            body: Block { terms: vec![] },
            ..Default::default()
        };
        assert_eq!(result, ast);
    }

    #[test]
    fn multiline_3() {
        let code = r"
/* 1 **/
";
        let result = parse(code).unwrap();
        let ast = Module {
            body: Block { terms: vec![] },
            ..Default::default()
        };
        assert_eq!(result, ast);
    }

    #[test]
    fn multiline_4() {
        let code = r"
/* 1 * * */
";
        let result = parse(code).unwrap();
        let ast = Module {
            body: Block { terms: vec![] },
            ..Default::default()
        };
        assert_eq!(result, ast);
    }

    #[test]
    fn multiline_5() {
        let code = r"
/* 1 * * 

*/
";
        let result = parse(code).unwrap();
        let ast = Module {
            body: Block { terms: vec![] },
            ..Default::default()
        };
        assert_eq!(result, ast);
    }

    #[test]
    fn multiline_6() {
        let code = r"
/*/ 1 * * 

*/
";
        let result = parse(code).unwrap();
        let ast = Module {
            body: Block { terms: vec![] },
            ..Default::default()
        };
        assert_eq!(result, ast);
    }

    #[test]
    fn multiline_7() {
        let code = r"
/*/ 1 * * 

*/4
";
        let result = parse(code).unwrap();
        let ast = Module {
            body: Block {
                terms: vec![4f64.into()],
            },
            ..Default::default()
        };
        assert_eq!(result, ast);
    }

    #[test]
    fn multiline_8() {
        let code = r"
5/*/ 1 * * 

*/
";
        let result = parse(code).unwrap();
        let ast = Module {
            body: Block {
                terms: vec![5f64.into()],
            },
            ..Default::default()
        };
        assert_eq!(result, ast);
    }

    #[test]
    fn bad_symbols_1() {
        let symbols = [
            Symbol::CurlyClose,
            Symbol::SquareClose,
            Symbol::ParenClose,
            Symbol::ParenOpen,
            Symbol::Colon,
        ];

        for symbol in symbols {
            let result = parse(&format!("{:?}", symbol));
            assert_eq!(
                result,
                Err(ParseError::Unexpected(UnexpectedError::GeneralSymbol(
                    symbol,
                    SourceLocation::start()
                )))
            );
        }
    }

    #[test]
    fn bad_symbols_2() {
        let symbols = [Symbol::Hash];

        for symbol in symbols {
            let result = parse(&format!("fn: {:?}", symbol));
            assert_eq!(
                result,
                Err(ParseError::Unexpected(UnexpectedError::GeneralSymbol(
                    symbol,
                    SourceLocation {
                        line: 0,
                        character: 4,
                        column: 4
                    },
                )))
            );
        }
    }
}
