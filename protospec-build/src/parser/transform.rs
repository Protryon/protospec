use super::*;

pub fn parse_condition_and_transform(
    t: &mut TokenIter,
) -> ParseResult<(Vec<Ident>, Option<Box<Expression>>, Vec<Transform>)> {
    let flags = parse_flags(t)?;

    let condition = parse_conditional_clause(t)?;

    let mut transforms = vec![];

    while t.eat(Token::Arrow).is_some() {
        let name = t.expect_ident()?;
        let mut span = name.span;
        let mut arguments = vec![];
        if t.eat(Token::LeftParen).is_some() {
            loop {
                if let Some(token) = t.eat(Token::RightParen) {
                    span = span + token.span;
                    break;
                }
                arguments.push(parse_expression(t)?);
                if t.eat(Token::Comma).is_none() {
                    span = span + t.expect(Token::RightParen)?;
                    break;
                }
            }
        }
        let conditional = parse_conditional_clause(t)?;
        if let Some(conditional) = &conditional {
            span = span + *conditional.span();
        }
        transforms.push(Transform {
            span,
            arguments,
            name,
            conditional,
        });
    }

    Ok((flags, condition, transforms))
}
