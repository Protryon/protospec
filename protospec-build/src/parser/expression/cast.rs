use super::*;

pub fn parse_cast_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_unary_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) = t.eat_any(&[Token::Cast, Token::Elvis]) {
        match op {
            Token::Cast => {
                let right = parse_type(t)?;
                expr = Expression::Cast(CastExpression {
                    span: *expr.span() + right.span,
                    inner: Box::new(expr),
                    type_: right,
                })
            }
            Token::Elvis => {
                let right = parse_unary_expression(t)?;
                expr = Expression::Binary(BinaryExpression {
                    span: *expr.span() + *right.span(),
                    op: BinaryOp::Elvis,
                    left: Box::new(expr),
                    right: Box::new(right),
                })
            }
            _ => unimplemented!(),
        }
    }
    Ok(expr)
}
