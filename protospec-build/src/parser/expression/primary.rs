use super::*;

pub fn parse_primary_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let SpannedToken { token, span } = t.expect_any()?;
    Ok(match token {
        Token::Int(value) => Expression::Int(Int {
            value,
            type_: parse_scalar_type(t).map(|x| x.scalar),
            span,
        }),
        Token::String(content) => Expression::Str(Str { content, span }),
        Token::True | Token::False => Expression::Bool(Bool {
            value: token == Token::True,
            span,
        }),
        Token::Ident(name) => {
            let ident = Ident { name, span };
            if t.eat(Token::DoubleColon).is_some() {
                let variant = t.expect_ident()?;
                Expression::EnumAccess(EnumAccessExpression {
                    span: ident.span + ident.span,
                    name: ident,
                    variant,
                })
            } else if t.peek_token(Token::LeftParen)? {
                let mut span = span;
                let arguments = parse_arguments(t, &mut span)?;
                Expression::Call(CallExpression {
                    function: ident,
                    arguments,
                    span,
                })
            } else {
                Expression::Ref(ident)
            }
        }
        Token::LeftParen => {
            let expr = parse_expression(t)?;
            t.expect(Token::RightParen)?;
            expr
        }
        token => {
            return Err(ParseError::Unexpected(
                SpannedToken { token, span },
                "expression".to_string(),
            ));
        }
    })
}
