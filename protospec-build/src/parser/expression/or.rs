use super::*;

pub fn parse_or_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_and_expression(t)?;
    while t.eat(Token::Or).is_some() {
        let right = parse_and_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: BinaryOp::Or,
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}
