use crate::ast::*;
use crate::result::*;
use crate::tokenizer::*;
use thiserror::Error;

pub type ParseResult<T> = StdResult<T, ParseError>;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("error tokenizing: `{0}`")]
    TokenError(String),
    #[error("unexpected eof")]
    UnexpectedEOF,
    #[error("unexpected token: {0}, expecting: {1}")]
    Unexpected(SpannedToken, String),
    #[error("length constraint cannot be empty")]
    EmptyLengthConstraint(Span),
    #[error("enum is missing representation scalar")]
    EnumMissingRep(Span),

    #[error("unknown")]
    Unknown(#[from] crate::Error),
}

struct TokenIter {
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

pub fn parse(script: &str) -> ParseResult<Program> {
    let mut tokens = TokenIter::new(crate::tokenize(script)?);

    parse_program(&mut tokens)
}

fn parse_program(t: &mut TokenIter) -> ParseResult<Program> {
    let mut declarations = vec![];
    while t.has_next() {
        declarations.push(parse_declaration(t)?);
    }
    Ok(Program { declarations })
}

fn parse_declaration(t: &mut TokenIter) -> ParseResult<Declaration> {
    let declaration = match t.peek()? {
        Token::Type => Declaration::Type(parse_type_declaration(t)?),
        Token::Import => Declaration::Import(parse_import_declaration(t)?),
        Token::ImportFfi => Declaration::Ffi(parse_ffi_declaration(t)?),
        Token::Const => Declaration::Const(parse_const_declaration(t)?),
        _ => {
            return Err(ParseError::Unexpected(
                t.expect_any()?,
                "'type', 'import', 'import_ffi', or 'const'".to_string(),
            ))
        }
    };
    t.expect(Token::Semicolon)?;
    Ok(declaration)
}

fn parse_type_declaration(t: &mut TokenIter) -> ParseResult<TypeDeclaration> {
    let start = t.expect(Token::Type)?;
    let name = t.expect_ident()?;
    let mut arguments = vec![];
    if t.eat(Token::LeftParen).is_some() {
        while t.eat(Token::RightParen).is_none() {
            let name = t.expect_ident()?;
            t.expect(Token::Colon)?;
            let inner_type = parse_type(t, true)?;

            let default_value = if t.eat(Token::Question).is_some() {
                Some(parse_expression(t)?)
            } else {
                None
            };
            arguments.push(TypeArgument {
                span: name.span,
                name,
                type_: inner_type,
                default_value,
            });
            if t.eat(Token::Comma).is_none() {
                t.expect(Token::RightParen)?;
                break;
            }
        }
    }

    t.expect(Token::Equal)?;
    let value = parse_field(t)?;

    Ok(TypeDeclaration {
        span: start + value.span,
        name,
        value,
        arguments,
    })
}

fn parse_import_declaration(t: &mut TokenIter) -> ParseResult<ImportDeclaration> {
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

fn parse_ffi_declaration(t: &mut TokenIter) -> ParseResult<FfiDeclaration> {
    let start = t.expect(Token::ImportFfi)?;
    let name = t.expect_ident()?;
    t.expect(Token::As)?;
    let typ = t.expect_oneof(&[Token::Transform, Token::Type, Token::Function])?;

    Ok(FfiDeclaration {
        span: start + typ.span,
        name,
        ffi_type: match typ.token {
            Token::Transform => FfiType::Transform,
            Token::Type => FfiType::Type,
            Token::Function => FfiType::Function,
            _ => unimplemented!(),
        },
    })
}

fn parse_const_declaration(t: &mut TokenIter) -> ParseResult<ConstDeclaration> {
    let start = t.expect(Token::Const)?;
    let name = t.expect_ident()?;
    t.expect(Token::Colon)?;
    let type_ = parse_type(t, true)?;
    t.expect(Token::Equal)?;
    let value = parse_expression(t)?;

    Ok(ConstDeclaration {
        span: start + *value.span(),
        name,
        type_,
        value,
    })
}

fn parse_scalar_type(t: &mut TokenIter) -> Option<ScalarType> {
    let SpannedToken { token, .. } = t
        .expect_oneof(&[
            Token::I8,
            Token::I16,
            Token::I32,
            Token::I64,
            Token::I128,
            Token::U8,
            Token::U16,
            Token::U32,
            Token::U64,
            Token::U128,
        ])
        .ok()?;
    Some(match token {
        Token::I8 => ScalarType::I8,
        Token::I16 => ScalarType::I16,
        Token::I32 => ScalarType::I32,
        Token::I64 => ScalarType::I64,
        Token::I128 => ScalarType::I128,
        Token::U8 => ScalarType::U8,
        Token::U16 => ScalarType::U16,
        Token::U32 => ScalarType::U32,
        Token::U64 => ScalarType::U64,
        Token::U128 => ScalarType::U128,
        _ => return None,
    })
}

fn parse_arguments(t: &mut TokenIter, span: &mut Span) -> ParseResult<Vec<Expression>> {
    let mut arguments = vec![];
    if t.eat(Token::LeftParen).is_some() {
        loop {
            if let Some(token) = t.eat(Token::RightParen) {
                *span = *span + token.span;
                break;
            }
            arguments.push(parse_expression(t)?);
            if t.eat(Token::Comma).is_none() {
                *span = *span + t.expect(Token::RightParen)?;
                break;
            }
        }
    }
    Ok(arguments)
}

fn parse_type(t: &mut TokenIter, direct_array: bool) -> ParseResult<Type> {
    let start = t.peek_span()?;

    let raw_type = match t.peek()? {
        Token::Container => RawType::Container(parse_container(t)?),
        Token::Enum => RawType::Enum(parse_enum(t)?),
        _ => {
            if let Some(scalar) = parse_scalar_type(t) {
                RawType::Scalar(scalar)
            } else {
                let SpannedToken { token, span } = t.expect_any()?;
                match token {
                    Token::F32 => RawType::F32,
                    Token::F64 => RawType::F64,
                    Token::Bool => RawType::Bool,
                    Token::Ident(name) => {
                        let name = Ident { name, span };
                        let mut span = name.span;
                        let arguments = parse_arguments(t, &mut span)?;
                        RawType::Ref(TypeCall {
                            name,
                            arguments,
                            span,
                        })
                    }
                    _ => {
                        return Err(ParseError::Unexpected(
                            SpannedToken { token, span },
                            "'container', 'enum', integer, float, 'bool', or identifier"
                                .to_string(),
                        ))
                    }
                }
            }
        }
    };

    let mut out = Type {
        span: start,
        raw_type,
    };

    if direct_array {
        while t.eat(Token::LeftSquare).is_some() {
            let length = parse_length_constraint(t)?;
            let end = t.expect(Token::RightSquare)?;
            out = Type {
                span: start + end,
                raw_type: RawType::Array(ArrayType {
                    element: Box::new(Field {
                        type_: out,
                        condition: None,
                        transforms: vec![],
                        span: start,
                        flags: vec![],
                    }),
                    length,
                    span: start + end,
                }),
            };
        }
    }

    Ok(out)
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

fn parse_flags(t: &mut TokenIter) -> ParseResult<Vec<Ident>> {
    let mut out = vec![];
    while t.eat(Token::Plus).is_some() {
        out.push(t.expect_ident()?);
    }
    Ok(out)
}

fn parse_condition_and_transform(
    t: &mut TokenIter,
) -> ParseResult<(Vec<Ident>, Option<Box<Expression>>, Vec<Transform>)> {
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

    Ok((flags, condition, transforms))
}

fn parse_field(t: &mut TokenIter) -> ParseResult<Field> {
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
                raw_type: RawType::Array(ArrayType {
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

fn parse_container(t: &mut TokenIter) -> ParseResult<Container> {
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

fn parse_length_constraint(t: &mut TokenIter) -> ParseResult<LengthConstraint> {
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

fn parse_enum(t: &mut TokenIter) -> ParseResult<Enum> {
    let start = t.expect(Token::Enum)?;

    let rep = parse_scalar_type(t).ok_or(ParseError::EnumMissingRep(start))?;

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

    Ok(Enum {
        span: start + end,
        rep,
        items,
    })
}

fn parse_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let expr = parse_or_expression(t)?;
    if t.eat(Token::Question).is_some() {
        let if_true = parse_expression(t)?;
        t.expect(Token::Colon)?;
        let if_false = parse_or_expression(t)?;
        Ok(Expression::Ternary(TernaryExpression {
            span: *expr.span() + *if_false.span(),
            condition: Box::new(expr),
            if_true: Box::new(if_true),
            if_false: Box::new(if_false),
        }))
    } else {
        Ok(expr)
    }
}

fn parse_or_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_and_expression(t)?;
    while t.eat(Token::Or).is_some() {
        let right = parse_and_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: BinaryOp::Or,
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}

fn parse_and_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_bit_or_expression(t)?;
    while t.eat(Token::And).is_some() {
        let right = parse_bit_or_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: BinaryOp::And,
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}

fn parse_bit_or_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_bit_xor_expression(t)?;
    while t.eat(Token::BitOr).is_some() {
        let right = parse_bit_xor_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: BinaryOp::BitOr,
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}

fn parse_bit_xor_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_bit_and_expression(t)?;
    while t.eat(Token::BitXor).is_some() {
        let right = parse_bit_and_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: BinaryOp::BitXor,
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}

fn parse_bit_and_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_eq_expression(t)?;
    while t.eat(Token::BitAnd).is_some() {
        let right = parse_eq_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: BinaryOp::BitAnd,
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}

fn parse_eq_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_rel_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) = t.eat_any(&[Token::Eq, Token::Ne]) {
        let right = parse_rel_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: match op {
                Token::Eq => BinaryOp::Eq,
                Token::Ne => BinaryOp::Ne,
                _ => unimplemented!(),
            },
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}

fn parse_rel_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_shift_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) =
        t.eat_any(&[Token::Lt, Token::LtEq, Token::Gt, Token::GtEq])
    {
        let right = parse_shift_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: match op {
                Token::Lt => BinaryOp::Lt,
                Token::LtEq => BinaryOp::Lte,
                Token::Gt => BinaryOp::Gt,
                Token::GtEq => BinaryOp::Gte,
                _ => unimplemented!(),
            },
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}

fn parse_shift_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_add_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) =
        t.eat_any(&[Token::Shl, Token::Shr, Token::ShrSigned])
    {
        let right = parse_add_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: match op {
                Token::Shl => BinaryOp::Shl,
                Token::Shr => BinaryOp::Shr,
                Token::ShrSigned => BinaryOp::ShrSigned,
                _ => unimplemented!(),
            },
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}

fn parse_add_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_multiply_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) = t.eat_any(&[Token::Plus, Token::Minus]) {
        let right = parse_multiply_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: match op {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => unimplemented!(),
            },
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}

fn parse_multiply_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_cast_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) =
        t.eat_any(&[Token::Mul, Token::Div, Token::Mod])
    {
        let right = parse_cast_expression(t)?;
        expr = Expression::Binary(BinaryExpression {
            span: *expr.span() + *right.span(),
            op: match op {
                Token::Mul => BinaryOp::Mul,
                Token::Div => BinaryOp::Div,
                Token::Mod => BinaryOp::Mod,
                _ => unimplemented!(),
            },
            left: Box::new(expr),
            right: Box::new(right),
        })
    }
    Ok(expr)
}

fn parse_cast_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_unary_expression(t)?;
    while let Some(SpannedToken { token: op, .. }) = t.eat_any(&[Token::Cast, Token::Elvis]) {
        match op {
            Token::Cast => {
                let right = parse_type(t, true)?;
                expr = Expression::Cast(CastExpression {
                    span: *expr.span() + right.span,
                    inner: Box::new(expr),
                    type_: right,
                })
            }
            Token::Elvis => {
                let right = parse_unary_expression(t)?;
                expr = Expression::Binary(BinaryExpression {
                    span: *expr.span() + *right.span(),
                    op: BinaryOp::Elvis,
                    left: Box::new(expr),
                    right: Box::new(right),
                })
            }
            _ => unimplemented!(),
        }
    }
    Ok(expr)
}

fn parse_unary_expression(t: &mut TokenIter) -> ParseResult<Expression> {
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

fn parse_array_index_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let mut expr = parse_primary_expression(t)?;
    while t.eat(Token::LeftSquare).is_some() {
        let right = parse_expression(t)?;
        let end = t.expect(Token::RightSquare)?;
        expr = Expression::ArrayIndex(ArrayIndexExpression {
            span: *expr.span() + end,
            array: Box::new(expr),
            index: Box::new(right),
        });
    }
    Ok(expr)
}

fn parse_primary_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let SpannedToken { token, span } = t.expect_any()?;
    Ok(match token {
        Token::Int(value) => Expression::Int(Int {
            value,
            type_: parse_scalar_type(t),
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
            } else {
                if t.peek_token(Token::LeftParen)? {
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
