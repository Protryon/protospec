use super::*;

pub fn parse_field(t: &mut TokenIter) -> ParseResult<Field> {
    let start = t.peek_span()?;

    let type_ = parse_type(t)?;
    let FieldComponents {
        calculated,
        flags,
        condition,
        transforms,
    } = parse_field_components(t)?;

    let out = Field {
        span: transforms
            .last()
            .map(|x| x.span + start)
            .or_else(|| condition.as_ref().map(|x| *x.span() + start))
            .unwrap_or(start),
        type_,
        calculated,
        condition,
        transforms,
        flags,
    };
    Ok(out)
}

struct FieldComponents {
    calculated: Option<Box<Expression>>,
    flags: Vec<Ident>,
    condition: Option<Box<Expression>>,
    transforms: Vec<Transform>,
}

fn parse_conditional_clause(t: &mut TokenIter) -> ParseResult<Option<Box<Expression>>> {
    Ok(if t.eat(Token::LeftCurly).is_some() {
        let condition = parse_expression(t)?;
        t.expect(Token::RightCurly)?;
        Some(Box::new(condition))
    } else {
        None
    })
}

fn parse_calculated_clause(t: &mut TokenIter) -> ParseResult<Option<Box<Expression>>> {
    Ok(if t.eat(Token::Equal).is_some() {
        let calculated = parse_expression(t)?;
        Some(Box::new(calculated))
    } else {
        None
    })
}

fn parse_field_components(t: &mut TokenIter) -> ParseResult<FieldComponents> {
    let calculated = parse_calculated_clause(t)?;

    let flags = parse_flags(t)?;

    let condition = parse_conditional_clause(t)?;

    let mut transforms = vec![];

    while t.eat(Token::Arrow).is_some() {
        let name = t.expect_ident()?;
        let mut span = name.span;
        let mut arguments = vec![];
        if t.eat(Token::LeftParen).is_some() {
            loop {
                if let Some(token) = t.eat(Token::RightParen) {
                    span = span + token.span;
                    break;
                }
                arguments.push(parse_expression(t)?);
                if t.eat(Token::Comma).is_none() {
                    span = span + t.expect(Token::RightParen)?;
                    break;
                }
            }
        }
        let conditional = parse_conditional_clause(t)?;
        if let Some(conditional) = &conditional {
            span = span + *conditional.span();
        }
        transforms.push(Transform {
            span,
            arguments,
            name,
            conditional,
        });
    }

    Ok(FieldComponents {
        calculated,
        flags,
        condition,
        transforms,
    })
}
