use super::*;

pub fn parse_scalar_type(t: &mut TokenIter) -> Option<ScalarType> {
    let SpannedToken { token, .. } = t
        .expect_oneof(&[
            Token::I8,
            Token::I16,
            Token::I32,
            Token::I64,
            Token::I128,
            Token::U8,
            Token::U16,
            Token::U32,
            Token::U64,
            Token::U128,
        ])
        .ok()?;
    Some(match token {
        Token::I8 => ScalarType::I8,
        Token::I16 => ScalarType::I16,
        Token::I32 => ScalarType::I32,
        Token::I64 => ScalarType::I64,
        Token::I128 => ScalarType::I128,
        Token::U8 => ScalarType::U8,
        Token::U16 => ScalarType::U16,
        Token::U32 => ScalarType::U32,
        Token::U64 => ScalarType::U64,
        Token::U128 => ScalarType::U128,
        _ => return None,
    })
}
