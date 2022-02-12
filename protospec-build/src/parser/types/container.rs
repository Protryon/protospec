use super::*;

pub fn parse_container(t: &mut TokenIter) -> ParseResult<Container> {
    let start = t.expect(Token::Container)?;

    let length = if t.eat(Token::LeftSquare).is_some() {
        let length = parse_expression(t)?;
        t.expect(Token::RightSquare)?;
        Some(Box::new(length))
    } else {
        None
    };

    let flags = parse_flags(t)?;

    t.expect(Token::LeftCurly)?;

    let mut items = vec![];

    loop {
        let ident = t.expect_ident()?;
        t.expect(Token::Colon)?;
        let type_ = parse_field(t)?;
        items.push((ident, type_));
        if !t.eat(Token::Comma).is_some() {
            break;
        }
        if t.peek_token(Token::RightCurly)? {
            break;
        }
    }

    let end = t.expect(Token::RightCurly)?;

    Ok(Container {
        span: start + end,
        length,
        flags,
        items,
    })
}
