use super::*;

pub struct TokenIter {
    inner: Vec<SpannedToken>,
}

impl Iterator for TokenIter {
    type Item = SpannedToken;

    fn next(&mut self) -> Option<SpannedToken> {
        self.inner.pop()
    }
}

impl TokenIter {
    pub fn new(mut tokens: Vec<SpannedToken>) -> Self {
        tokens.reverse();
        TokenIter { inner: tokens }
    }

    pub fn peek(&self) -> ParseResult<&Token> {
        self.inner
            .last()
            .map(|x| &x.token)
            .ok_or(ParseError::UnexpectedEOF)
    }

    pub fn peek_token(&self, token: Token) -> ParseResult<bool> {
        self.inner
            .last()
            .map(|x| &x.token == &token)
            .ok_or(ParseError::UnexpectedEOF)
    }

    pub fn peek_span(&self) -> ParseResult<Span> {
        self.inner
            .last()
            .map(|x| x.span)
            .ok_or(ParseError::UnexpectedEOF)
    }

    pub fn has_next(&self) -> bool {
        self.inner.len() > 0
    }

    pub fn eat(&mut self, token: Token) -> Option<SpannedToken> {
        if let Some(SpannedToken { token: inner, .. }) = self.inner.last() {
            if &token == inner {
                return self.inner.pop();
            }
        }
        None
    }

    pub fn eat_any(&mut self, token: &[Token]) -> Option<SpannedToken> {
        if let Some(SpannedToken { token: inner, .. }) = self.inner.last() {
            if token.iter().any(|x| x == inner) {
                return self.inner.pop();
            }
        }
        None
    }

    pub fn expect(&mut self, token: Token) -> ParseResult<Span> {
        if let Some(SpannedToken { token: inner, span }) = self.inner.last() {
            if &token == inner {
                return Ok(self.inner.pop().unwrap().span);
            } else {
                return Err(ParseError::Unexpected(
                    SpannedToken {
                        token: inner.clone(),
                        span: *span,
                    },
                    token.to_string().trim().to_string(),
                ));
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }

    pub fn expect_oneof(&mut self, token: &[Token]) -> ParseResult<SpannedToken> {
        if let Some(SpannedToken { token: inner, span }) = self.inner.last() {
            if token.iter().any(|x| x == inner) {
                return Ok(self.inner.pop().unwrap());
            } else {
                return Err(ParseError::Unexpected(
                    SpannedToken {
                        token: inner.clone(),
                        span: *span,
                    },
                    token
                        .iter()
                        .map(|x| format!("'{}'", x.to_string().trim()))
                        .collect::<Vec<_>>()
                        .join(", "),
                ));
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }

    pub fn expect_ident(&mut self) -> ParseResult<Ident> {
        if let Some(SpannedToken { token: inner, span }) = self.inner.last() {
            if let Token::Ident(_) = inner {
                let token = self.inner.pop().unwrap();
                if let SpannedToken {
                    token: Token::Ident(name),
                    span,
                } = token
                {
                    Ok(Ident { name, span })
                } else {
                    unimplemented!()
                }
            } else {
                return Err(ParseError::Unexpected(
                    SpannedToken {
                        token: inner.clone(),
                        span: *span,
                    },
                    "ident".to_string(),
                ));
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }

    pub fn expect_string(&mut self) -> ParseResult<Str> {
        if let Some(SpannedToken { token: inner, span }) = self.inner.last() {
            if let Token::String(_) = inner {
                let token = self.inner.pop().unwrap();
                if let SpannedToken {
                    token: Token::String(content),
                    span,
                } = token
                {
                    Ok(Str { content, span })
                } else {
                    unimplemented!()
                }
            } else {
                return Err(ParseError::Unexpected(
                    SpannedToken {
                        token: inner.clone(),
                        span: *span,
                    },
                    "string".to_string(),
                ));
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }

    pub fn expect_any(&mut self) -> ParseResult<SpannedToken> {
        if let Some(x) = self.inner.pop() {
            Ok(x)
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }
}
