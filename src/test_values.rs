#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::lang::{OwnedValue, Value, string::CharString};

    #[test]
    fn i32() {
        assert_eq!(Value::from(32i32), Value::Number(32.));
        assert_eq!(OwnedValue::from(32i32), OwnedValue::Number(32.));
    }

    #[test]
    fn debug() {
        assert_eq!("32", &format!("{:?}", Value::Number(32.)));
        assert_eq!(
            r#""a\n""#,
            &format!("{:?}", Value::String(Rc::new("a\n".into())))
        );
        assert_eq!(
            "Fn[3, test]",
            &format!("{:?}", Value::Address(3, "test".into()))
        );
        assert_eq!("32", &format!("{:?}", OwnedValue::Number(32.)));
        assert_eq!(
            r#""a\n""#,
            &format!("{:?}", OwnedValue::String("a\n".into()))
        );
        assert_eq!(
            "Fn[3, test]",
            &format!("{:?}", OwnedValue::Address(3, "test".into()))
        );
    }

    #[test]
    fn display() {
        assert_eq!("32", &format!("{}", Value::Number(32.)));
        assert_eq!("a\n", &format!("{}", Value::String(Rc::new("a\n".into()))));
        assert_eq!(
            "Fn[3, test]",
            &format!("{}", Value::Address(3, "test".into()))
        );
    }

    #[test]
    fn convert() {
        assert_eq!("😃a😄b😁", &CharString::from("😃a😄b😁").to_string());
    }
}
