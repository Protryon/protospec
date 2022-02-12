use super::*;

pub fn parse_add_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_multiply_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) = t.eat_any(&[Token::Plus, Token::Minus]) {
        let right = parse_multiply_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: match op {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => unimplemented!(),
            },
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}
