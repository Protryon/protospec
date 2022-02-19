use super::*;

pub fn parse_bitfield(t: &mut TokenIter) -> ParseResult<Bitfield> {
    let start = t.expect(Token::Bitfield)?;

    let rep = parse_scalar_type(t).ok_or(ParseError::BitfieldMissingRep(start))?;

    t.expect(Token::LeftCurly)?;

    let mut items = vec![];

    loop {
        let ident = t.expect_ident()?;
        let expr = if items.len() == 0 {
            t.expect(Token::Equal)?;
            Some(Box::new(parse_expression(t)?))
        } else {
            if t.eat(Token::Equal).is_some() {
                Some(Box::new(parse_expression(t)?))
            } else {
                None
            }
        };

        items.push((ident, expr));
        if !t.eat(Token::Comma).is_some() {
            break;
        }
        if t.peek_token(Token::RightCurly)? {
            break;
        }
    }

    let end = t.expect(Token::RightCurly)?;

    Ok(Bitfield {
        span: start + end,
        rep,
        items,
    })
}
