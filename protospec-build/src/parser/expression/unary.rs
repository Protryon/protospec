use super::*;

pub fn parse_unary_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut ops = vec![];
    while let Some(token) = t.eat_any(&[Token::Not, Token::Minus, Token::BitNot]) {
        ops.push(token);
    }
    let mut inner = parse_array_index_expression(t)?;
    for op in ops.into_iter().rev() {
        inner = Expression::Unary(UnaryExpression {
            span: op.span + *inner.span(),
            op: match op.token {
                Token::Not => UnaryOp::Not,
                Token::Minus => UnaryOp::Negate,
                Token::BitNot => UnaryOp::BitNot,
                _ => unimplemented!(),
            },
            inner: Box::new(inner),
        });
    }
    Ok(inner)
}
