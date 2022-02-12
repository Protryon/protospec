use super::*;

pub fn parse_and_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_bit_or_expression(t)?;
    while t.eat(Token::And).is_some() {
        let right = parse_bit_or_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: BinaryOp::And,
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}
