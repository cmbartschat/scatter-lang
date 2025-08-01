#[cfg(test)]
mod tests {
    use crate::lang::Value;
    use crate::{interpreter::Interpreter, parser::parse};

    fn interpret_str(source: &str) -> Vec<Value> {
        let ast = parse(source).unwrap();
        let mut ctx = Interpreter::new();
        ctx.load(&ast).expect("Execution error");
        ctx.stack
    }

    #[test]
    fn math() {
        assert_eq!(
            interpret_str(r##"1 1 + 0 "hello" 0 -3 -3 - 0 =="##),
            vec![2.into(), 0.into(), "hello".into(), 0.into(), true.into()]
        )
    }
}
