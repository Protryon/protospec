use super::*;

pub fn parse_type_declaration(t: &mut TokenIter) -> ParseResult<TypeDeclaration> {
    let start = t.expect(Token::Type)?;
    let name = t.expect_ident()?;
    let mut arguments = vec![];
    if t.eat(Token::LeftParen).is_some() {
        while t.eat(Token::RightParen).is_none() {
            let name = t.expect_ident()?;
            t.expect(Token::Colon)?;
            let inner_type = parse_type(t, true)?;

            let default_value = if t.eat(Token::Question).is_some() {
                Some(parse_expression(t)?)
            } else {
                None
            };
            arguments.push(TypeArgument {
                span: name.span,
                name,
                type_: inner_type,
                default_value,
            });
            if t.eat(Token::Comma).is_none() {
                t.expect(Token::RightParen)?;
                break;
            }
        }
    }

    t.expect(Token::Equal)?;
    let value = parse_field(t)?;

    Ok(TypeDeclaration {
        span: start + value.span,
        name,
        value,
        arguments,
    })
}
