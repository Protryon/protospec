use super::*;

pub fn parse_array_index_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_primary_expression(t)?;
    while t.eat(Token::LeftSquare).is_some() {
        let right = parse_expression(t)?;
        let end = t.expect(Token::RightSquare)?;
        expr = Expression::ArrayIndex(ArrayIndexExpression {
            span: *expr.span() + end,
            array: Box::new(expr),
            index: Box::new(right),
        });
    }
    Ok(expr)
}
