use super::*;

pub fn parse_ffi_declaration(t: &mut TokenIter) -> ParseResult<FfiDeclaration> {
    let start = t.expect(Token::ImportFfi)?;
    let name = t.expect_ident()?;
    t.expect(Token::As)?;
    let typ = t.expect_oneof(&[Token::Transform, Token::Type, Token::Function])?;

    Ok(FfiDeclaration {
        span: start + typ.span,
        name,
        ffi_type: match typ.token {
            Token::Transform => FfiType::Transform,
            Token::Type => FfiType::Type,
            Token::Function => FfiType::Function,
            _ => unimplemented!(),
        },
    })
}
