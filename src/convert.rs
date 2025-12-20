static MAX_SAFE_F64: f64 = 9007199254740991.;
static MAX_SAFE_USIZE: usize = MAX_SAFE_F64 as usize;

pub fn f64_to_usize(v: f64) -> Option<usize> {
    if !v.is_finite() {
        return None; // Nan/Infinite
    }
    if v > MAX_SAFE_F64 {
        return None; // Too big for mantissa
    }
    if v < 0f64 {
        return None; // Negative
    }
    if v.fract() != 0f64 {
        return None; // Not an integer
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Some(v as usize)
}

pub fn usize_to_f64(v: usize) -> Option<f64> {
    if v > MAX_SAFE_USIZE {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    Some(v as f64)
}

pub fn f64_to_char(v: f64) -> Option<char> {
    let v = f64_to_usize(v)?;
    if v > u8::MAX as usize {
        return None;
    }
    #[allow(clippy::cast_possible_truncation)]
    char::from_u32(v as u32)
}

pub fn hex_char_to_u8(c: char) -> Option<u8> {
    let v = c as u32;
    if v >= '0' as u32 && v <= '9' as u32 {
        return Some((v - '0' as u32) as u8);
    }

    if v >= 'a' as u32 && v <= 'f' as u32 {
        return Some((v - 'a' as u32) as u8 + 10);
    }

    if v >= 'A' as u32 && v <= 'F' as u32 {
        return Some((v - 'A' as u32) as u8 + 10);
    }

    None
}
