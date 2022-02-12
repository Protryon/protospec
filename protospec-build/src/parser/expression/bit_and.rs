use super::*;

pub fn parse_bit_and_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_eq_expression(t)?;
    while t.eat(Token::BitAnd).is_some() {
        let right = parse_eq_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: BinaryOp::BitAnd,
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}
