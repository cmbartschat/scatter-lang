#[cfg(test)]
mod tests {
    use crate::lang::{SourceLocation, Token};
    use crate::tokenizer::{EscapeSequenceError, TokenizeError, tokenize};

    fn expect_tokens(source: &str, expected: &[Token]) {
        let actual: Vec<Token> = tokenize(source)
            .unwrap()
            .into_iter()
            .map(|f| f.value)
            .collect();
        assert_eq!(actual, expected);
    }

    fn expect_error(source: &str, expected: &TokenizeError) {
        assert_eq!(&tokenize(source).unwrap_err(), expected);
    }

    #[test]
    fn string_1() {
        let source = r#""\t\r\n""#;
        let expected: Vec<Token> = vec![Token::String("\t\r\n".into())];
        expect_tokens(source, &expected);
    }

    #[test]
    fn string_2() {
        let source = r#""\1""#;
        let expected = TokenizeError::InvalidEscape(
            EscapeSequenceError::InvalidCharacter,
            SourceLocation {
                line: 0,
                character: 2,
                column: 2,
            },
        );
        expect_error(source, &expected);
    }

    #[test]
    fn string_3() {
        let source = r#""\xf-""#;
        let expected = TokenizeError::InvalidEscape(
            EscapeSequenceError::InvalidHex,
            SourceLocation {
                line: 0,
                character: 4,
                column: 4,
            },
        );
        expect_error(source, &expected);
    }

    #[test]
    fn string_4() {
        let source = r"'a'";
        let expected: Vec<Token> = vec![Token::String("a".into())];
        expect_tokens(source, &expected);
    }

    #[test]
    fn string_5() {
        let source = r"'a' test";
        let expected: Vec<Token> = vec![Token::String("a".into()), Token::Name("test".into())];
        expect_tokens(source, &expected);
    }

    #[test]
    fn normal_1() {
        let source = "a";
        let expected: Vec<Token> = vec![Token::Name("a".into())];
        expect_tokens(source, &expected);
    }

    #[test]
    fn unbounded_1() {
        let source = "'a";
        let expected = TokenizeError::UnboundedString(SourceLocation {
            line: 0,
            character: 0,
            column: 0,
        });
        expect_error(source, &expected);
    }

    #[test]
    fn unbounded_2() {
        let source = "/* a";
        let expected = TokenizeError::UnboundedComment(SourceLocation {
            line: 0,
            character: 0,
            column: 0,
        });
        expect_error(source, &expected);
    }

    #[test]
    fn empty_1() {
        let source = "";
        let expected: Vec<Token> = vec![];
        expect_tokens(source, &expected);
    }

    #[test]
    fn empty_2() {
        let source = "// test";
        let expected: Vec<Token> = vec![];
        expect_tokens(source, &expected);
    }

    #[test]
    fn empty_3() {
        let source = "// test\n";
        let expected: Vec<Token> = vec![];
        expect_tokens(source, &expected);
    }
}
