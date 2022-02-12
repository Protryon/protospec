use super::*;

pub fn parse_eq_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_rel_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) = t.eat_any(&[Token::Eq, Token::Ne]) {
        let right = parse_rel_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: match op {
                Token::Eq => BinaryOp::Eq,
                Token::Ne => BinaryOp::Ne,
                _ => unimplemented!(),
            },
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}
