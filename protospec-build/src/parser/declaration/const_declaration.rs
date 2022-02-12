use super::*;

pub fn parse_const_declaration(t: &mut TokenIter) -> ParseResult<ConstDeclaration> {
    let start = t.expect(Token::Const)?;
    let name = t.expect_ident()?;
    t.expect(Token::Colon)?;
    let type_ = parse_type(t, true)?;
    t.expect(Token::Equal)?;
    let value = parse_expression(t)?;

    Ok(ConstDeclaration {
        span: start + *value.span(),
        name,
        type_,
        value,
    })
}
