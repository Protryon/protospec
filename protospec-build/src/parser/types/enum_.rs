use super::*;

fn parse_enum_value(t: &mut TokenIter, first: bool) -> ParseResult<EnumValue> {
    if first {
        t.expect(Token::Equal)?;
    } else if t.eat(Token::Equal).is_none() {
        return Ok(EnumValue::None);
    }
    if t.eat(Token::Default).is_some() {
        Ok(EnumValue::Default)
    } else {
        Ok(EnumValue::Expression(Box::new(parse_expression(t)?)))
    }
}

pub fn parse_enum(t: &mut TokenIter) -> ParseResult<Enum> {
    let start = t.expect(Token::Enum)?;

    let rep = parse_scalar_type(t).ok_or(ParseError::EnumMissingRep(start))?;

    t.expect(Token::LeftCurly)?;

    let mut items = vec![];

    loop {
        let ident = t.expect_ident()?;
        let value = parse_enum_value(t, items.is_empty())?;

        items.push((ident, value));
        if !t.eat(Token::Comma).is_some() {
            break;
        }
        if t.peek_token(Token::RightCurly)? {
            break;
        }
    }

    let end = t.expect(Token::RightCurly)?;

    Ok(Enum {
        span: start + end,
        rep,
        items,
    })
}
