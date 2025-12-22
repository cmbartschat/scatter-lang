#[cfg(test)]
mod tests {
    use crate::convert::{f64_to_char, f64_to_usize, hex_char_to_u8, usize_to_f64};

    static MAX_SAFE_USIZE: usize = 0x1fffffffffffff;
    static MAX_SAFE_F64: f64 = 9007199254740991f64;

    #[test]
    fn f64_to_usize_invalid() {
        assert_eq!(f64_to_usize(f64::INFINITY), None);
        assert_eq!(f64_to_usize(-1.), None);
        assert_eq!(f64_to_usize(-10.), None);
        assert_eq!(f64_to_usize(1.5), None);
        assert_eq!(f64_to_usize(1e53), None);
        assert_eq!(f64_to_usize(f64::NAN), None);
        assert_eq!(f64_to_usize(f64::NEG_INFINITY), None);
        assert_eq!(f64_to_usize(MAX_SAFE_F64 + 1f64), None);
    }

    #[test]
    fn f64_to_usize_valid() {
        assert_eq!(f64_to_usize(0.).unwrap(), 0);
        assert_eq!(f64_to_usize(-0.).unwrap(), 0);
        assert_eq!(f64_to_usize(5.).unwrap(), 5);
        assert_eq!(f64_to_usize(MAX_SAFE_F64).unwrap(), MAX_SAFE_USIZE);
    }

    #[test]
    fn usize_to_f64_invalid() {
        assert_eq!(usize_to_f64(MAX_SAFE_USIZE + 1), None);
        assert_eq!(usize_to_f64(usize::MAX), None);
    }

    #[test]
    fn usize_to_f64_valid() {
        assert_eq!(usize_to_f64(0).unwrap(), 0.);
        assert_eq!(usize_to_f64(5).unwrap(), 5.);
        assert_eq!(usize_to_f64(MAX_SAFE_USIZE).unwrap(), MAX_SAFE_F64);
    }

    #[test]
    fn f64_to_char_invalid() {
        assert_eq!(f64_to_char(f64::INFINITY), None);
        assert_eq!(f64_to_char(-1.), None);
        assert_eq!(f64_to_char(-10.), None);
        assert_eq!(f64_to_char(1.5), None);
        assert_eq!(f64_to_char(1e53), None);
        assert_eq!(f64_to_char(f64::NAN), None);
        assert_eq!(f64_to_char(f64::NEG_INFINITY), None);
        assert_eq!(f64_to_char(1114112.), None);
    }

    #[test]
    fn f64_to_char_valid() {
        assert_eq!(f64_to_char(0.).unwrap(), '\0');
        assert_eq!(f64_to_char(65.).unwrap(), 'A');
        assert_eq!(f64_to_char(97.).unwrap(), 'a');
        assert_eq!(f64_to_char(255.).unwrap(), 0xff as char);
        assert_eq!(f64_to_char(1114111.).unwrap(), '\u{10FFFF}');
        assert_eq!(f64_to_char(9989.).unwrap(), '✅');
    }

    #[test]
    fn hex_char_to_u8_invalid() {
        assert_eq!(hex_char_to_u8('g'), None);
        assert_eq!(hex_char_to_u8('\0'), None);
        assert_eq!(hex_char_to_u8('G'), None);
        assert_eq!(hex_char_to_u8('!'), None);
        assert_eq!(hex_char_to_u8('$'), None);
        assert_eq!(hex_char_to_u8('x'), None);
        assert_eq!(hex_char_to_u8(' '), None);
    }

    #[test]
    fn hex_char_to_u8_valid() {
        assert_eq!(hex_char_to_u8('0').unwrap(), 0);
        assert_eq!(hex_char_to_u8('9').unwrap(), 9);
        assert_eq!(hex_char_to_u8('a').unwrap(), 10);
        assert_eq!(hex_char_to_u8('A').unwrap(), 10);
        assert_eq!(hex_char_to_u8('f').unwrap(), 15);
        assert_eq!(hex_char_to_u8('F').unwrap(), 15);
    }
}
