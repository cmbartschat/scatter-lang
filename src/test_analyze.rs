#[cfg(test)]
mod tests {

    use crate::analyze::{AnalysisError, NamespaceArities, analyze_program};

    use crate::lang::Module;
    use crate::parser::parse;
    use crate::program::Program;

    struct SimpleAnalysis {
        arities: NamespaceArities,
    }

    fn analyze(ast: &Module) -> SimpleAnalysis {
        let all = analyze_program(&Program::new_from_module(ast));
        SimpleAnalysis {
            arities: all.into_iter().next().unwrap(),
        }
    }

    fn assert_fn_arity(code: &str, expected_arity: &str) {
        let ast = parse(code).unwrap();
        let a = analyze(&ast);
        let actual_arity = a.arities["fn"].as_ref().unwrap().stringify();
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
        assert_eq!(&a.arities["fn"], &Err(expected_error));
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
    fn branch_5() {
        assert_fn_arity(
            r"
fn: {
  {
    (1) true
    (dup) - - - -
    (drop drop drop)
  }
}",
            "- b",
        );
    }

    #[test]
    fn branch_6() {
        assert_fn_arity(
            r"
fn: {
  {
    (0) true
    (0) - - - -
    (0)
  }
}",
            "-",
        );
    }

    #[test]
    fn branch_7() {
        assert_fn_arity(
            r#"
fn: {
  {
    (0) true
    (1) "hi"
    (0)
  }
}"#,
            "- s",
        );
    }

    #[test]
    fn branch_8() {
        assert_fn_arity(
            r#"
fn: {
  {
    (dup) true
    (1) "hi"
    (0)
  }
}"#,
            "0 - 0 u",
        );
    }

    #[test]
    fn branch_9() {
        assert_fn_arity(
            r#"
fn: {
  {
    (1 1 -) true
    (1) "hi"
    (0)
  }
}"#,
            "- u",
        );
    }

    #[test]
    fn branch_10() {
        assert_fn_err(
            r#"
fn: {
  {
    (1 1 -) true
    (0) "hi"
    (0)
    (1 1 -) false
  }
}"#,
            AnalysisError::IndefiniteSize,
        );
    }

    #[test]
    fn branch_11() {
        assert_fn_arity(
            r#"
fn: {
  {
    ("") 1
    (0 1) "string"
    (1) 1 1 1
  }
}"#,
            "- n s",
        );
    }

    #[test]
    fn branch_12() {
        assert_fn_arity(
            r#"
fn: {
  {
    ("") 1
    ("a") 1 1
    (1) 1 1 1
  }
}"#,
            "- n n",
        );
    }

    #[test]
    fn branch_13() {
        assert_fn_arity(
            r#"
fn: {
  {
    ("") 1
    (true) 1 1
    (1) 1 1 1
  }
}"#,
            "- n n",
        );
    }

    #[test]
    fn branch_14() {
        assert_fn_err(
            r"
fn: {
  {
    () 0 0 substring
    (1) 0 +
  }
}",
            AnalysisError::IncompatibleTypes,
        );
    }

    #[test]
    fn branch_15() {
        assert_fn_unset(
            r"
fn1: fn1
fn: {
  {
    () 0 0 substring
    (1) fn1 0 +
  }
}",
        );
    }

    #[test]
    fn branch_16() {
        assert_fn_arity(
            r"
fn1: 3

fn: {
  {
    (false) substring
    (@fn1) fn1 0 +
  }
}",
            "- n",
        );
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
    fn loop_5() {
        assert_fn_err("fn: [1]", AnalysisError::IndefiniteSize);
    }

    #[test]
    fn loop_6() {
        assert_fn_err("fn: [() 1 1 1 ()]", AnalysisError::IndefiniteSize);
    }

    #[test]
    fn loop_7() {
        assert_fn_arity("fn: [1 ()]", "-");
    }

    #[test]
    fn loop_8() {
        assert_fn_arity("fn: [(readline) print] drop", "-");
    }

    #[test]
    fn loop_9() {
        assert_fn_err(
            "fn: [(readline) print (readline)]",
            AnalysisError::IndefiniteSize,
        );
    }

    #[test]
    fn loop_10() {
        assert_fn_err(
            "fn: [(readline) print (readline drop)] drop",
            AnalysisError::IndefiniteSize,
        );
    }

    #[test]
    fn loop_11() {
        assert_fn_err(
            "fn: [(readline) print (1)] drop",
            AnalysisError::IndefiniteSize,
        );
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

    #[test]
    fn test_generic_6() {
        assert_fn_arity(
            r#"
fn: {
  ==
  {
    ()  "pass " join
    (1) "fail " join
  }
}
     "#,
            "u u u - s",
        );
    }

    #[test]
    fn test_generic_7() {
        assert_fn_arity(
            r"
fn: {
  swap ++ swap
}
     ",
            "n 0 - n 0",
        );
    }

    #[test]
    fn test_generic_8() {
        assert_fn_arity(
            r#"
fail: {
  swap ++ swap
}

fn: {
  ==
  {
    ()  "pass " join
    (1) "fail " join fail
  }
}
     "#,
            "n u u u - n s",
        );
    }

    #[test]
    fn test_generic_9() {
        assert_fn_arity(
            r#"
fail: {
  swap ++ swap
}

fn: {
  ==
  {
    (0)  "pass " join
    (1) "fail " join fail
  }
}
     "#,
            "n u u - n s",
        );
    }

    #[test]
    fn test_generic_10() {
        assert_fn_arity(
            r#"
fail: {
  swap ++ swap
}

fn: {
  1
  {
    ()  "pass " join
    (1) "fail " join fail
  }
}
     "#,
            "n u - n s",
        );
    }

    #[test]
    fn unresolved_1() {
        assert_fn_unset(
            r"
fn: {
  1
  fn
  drop
}
     ",
        );
    }

    #[test]
    fn unresolved_2() {
        assert_fn_unset(
            r"
fn1: fn

fn: {
  1
  fn1
  +
}
     ",
        );
    }

    #[test]
    fn unresolved_3() {
        assert_fn_unset(
            r"
fn1: fn1

fn: {
  [(fn1) print]
}
     ",
        );
    }

    #[test]
    fn unresolved_4() {
        assert_fn_unset(
            r"
fn: {
  [(fn1) print]
}
     ",
        );
    }

    #[test]
    fn intrinsic_1() {
        assert_fn_arity(
            r"
fn: substring
     ",
            "s n n - s",
        );
    }

    #[test]
    fn intrinsic_2() {
        assert_fn_arity(
            r"
fn: 3 substring
     ",
            "s n - s",
        );
    }

    #[test]
    fn intrinsic_3() {
        assert_fn_arity(
            r"
fn: 0 3 substring
     ",
            "s - s",
        );
    }
}
