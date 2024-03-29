use super::*;

pub fn parse_scalar_type(t: &mut TokenIter) -> Option<EndianScalarType> {
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
            Token::I16Le,
            Token::I32Le,
            Token::I64Le,
            Token::I128Le,
            Token::U16Le,
            Token::U32Le,
            Token::U64Le,
            Token::U128Le,
        ])
        .ok()?;
    let scalar = match token {
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
        Token::I16Le => ScalarType::I16,
        Token::I32Le => ScalarType::I32,
        Token::I64Le => ScalarType::I64,
        Token::I128Le => ScalarType::I128,
        Token::U16Le => ScalarType::U16,
        Token::U32Le => ScalarType::U32,
        Token::U64Le => ScalarType::U64,
        Token::U128Le => ScalarType::U128,
        _ => return None,
    };
    Some(EndianScalarType {
        scalar,
        endian: match token {
            Token::I16Le => Endian::Little,
            Token::I32Le => Endian::Little,
            Token::I64Le => Endian::Little,
            Token::I128Le => Endian::Little,
            Token::U16Le => Endian::Little,
            Token::U32Le => Endian::Little,
            Token::U64Le => Endian::Little,
            Token::U128Le => Endian::Little,
            _ => Endian::Big,
        },
    })
}
