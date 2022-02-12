use super::*;

mod scalar;
pub use scalar::parse_scalar_type;

mod container;
use container::*;

mod array;
pub use array::parse_length_constraint;

mod enum_;
use enum_::*;

pub fn parse_type(t: &mut TokenIter, direct_array: bool) -> ParseResult<Type> {
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
                        RawType::Ref(TypeRef {
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
                raw_type: RawType::Array(Array {
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
