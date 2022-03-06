use super::*;

pub fn parse_array_index_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_primary_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) = t.eat_any(&[Token::LeftSquare, Token::Dot]) {
        match op {
            Token::LeftSquare => {
                let right = parse_expression(t)?;
                let end = t.expect(Token::RightSquare)?;
                expr = Expression::ArrayIndex(ArrayIndexExpression {
                    span: *expr.span() + end,
                    array: Box::new(expr),
                    index: Box::new(right),
                });
            }
            Token::Dot => {
                let member = t.expect_ident()?;
                expr = Expression::Member(MemberExpression {
                    span: *expr.span() + member.span,
                    target: Box::new(expr),
                    member,
                })
            }
            _ => unimplemented!(),
        }
    }

    while t.eat(Token::LeftSquare).is_some() {}
    Ok(expr)
}
