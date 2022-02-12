use super::*;

pub fn parse_rel_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_shift_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) =
        t.eat_any(&[Token::Lt, Token::LtEq, Token::Gt, Token::GtEq])
    {
        let right = parse_shift_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: match op {
                Token::Lt => BinaryOp::Lt,
                Token::LtEq => BinaryOp::Lte,
                Token::Gt => BinaryOp::Gt,
                Token::GtEq => BinaryOp::Gte,
                _ => unimplemented!(),
            },
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}
