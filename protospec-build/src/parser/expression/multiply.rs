use super::*;

pub fn parse_multiply_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_cast_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) =
        t.eat_any(&[Token::Mul, Token::Div, Token::Mod])
    {
        let right = parse_cast_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: match op {
                Token::Mul => BinaryOp::Mul,
                Token::Div => BinaryOp::Div,
                Token::Mod => BinaryOp::Mod,
                _ => unimplemented!(),
            },
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}
