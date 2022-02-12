use super::*;

pub fn parse_bit_xor_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_bit_and_expression(t)?;
    while t.eat(Token::BitXor).is_some() {
        let right = parse_bit_and_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: BinaryOp::BitXor,
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}
