use super::*;

pub fn parse_shift_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_add_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) =
        t.eat_any(&[Token::Shl, Token::Shr, Token::ShrSigned])
    {
        let right = parse_add_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: match op {
                Token::Shl => BinaryOp::Shl,
                Token::Shr => BinaryOp::Shr,
                Token::ShrSigned => BinaryOp::ShrSigned,
                _ => unimplemented!(),
            },
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}
