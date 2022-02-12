use super::*;

pub fn parse_import_declaration(t: &mut TokenIter) -> ParseResult<ImportDeclaration> {
    let start = t.expect(Token::Import)?;
    let mut items = vec![];
    loop {
        let name = t.expect_ident()?;
        let alias = if let Some(_) = t.eat(Token::As) {
            Some(t.expect_ident()?)
        } else {
            None
        };
        items.push(ImportItem {
            span: alias
                .as_ref()
                .map(|x| x.span + name.span)
                .unwrap_or(name.span),
            name,
            alias,
        });
        if !t.eat(Token::Comma).is_some() {
            break;
        }
        if t.peek_token(Token::From)? {
            break;
        }
    }
    t.expect(Token::From)?;

    let from = t.expect_string()?;

    Ok(ImportDeclaration {
        span: start + from.span,
        items,
        from,
    })
}
