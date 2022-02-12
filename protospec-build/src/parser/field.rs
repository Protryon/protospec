use super::*;

pub fn parse_field(t: &mut TokenIter) -> ParseResult<Field> {
    let start = t.peek_span()?;

    let type_ = parse_type(t, false)?;
    let (flags, condition, transforms) = parse_condition_and_transform(t)?;

    let mut out = Field {
        span: transforms
            .last()
            .map(|x| x.span + start)
            .or_else(|| condition.as_ref().map(|x| *x.span() + start))
            .unwrap_or(start),
        type_,
        condition,
        transforms,
        flags,
    };
    while t.eat(Token::LeftSquare).is_some() {
        let length = parse_length_constraint(t)?;
        let end = t.expect(Token::RightSquare)?;

        let (flags, condition, transforms) = parse_condition_and_transform(t)?;
        out = Field {
            type_: Type {
                span: out.span + end,
                raw_type: RawType::Array(Array {
                    span: out.span + end,
                    element: Box::new(out),
                    length,
                }),
            },
            condition,
            transforms,
            flags,
            span: start + end,
        };
    }

    Ok(out)
}
