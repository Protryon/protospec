use super::*;

pub fn parse_length_constraint(t: &mut TokenIter) -> ParseResult<LengthConstraint> {
    let start = t.eat(Token::DotDot);
    let expression = if t.peek_token(Token::RightSquare)? {
        None
    } else {
        Some(Box::new(parse_expression(t)?))
    };

    if start.is_none() && expression.is_none() {
        return Err(ParseError::EmptyLengthConstraint(t.peek_span()?));
    }

    Ok(LengthConstraint {
        span: start
            .as_ref()
            .map(|x| x.span)
            .or_else(|| expression.as_ref().map(|x| *x.span()))
            .unwrap(),
        expandable: start.is_some(),
        inner: expression,
    })
}
