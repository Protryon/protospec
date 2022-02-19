use thiserror::Error;
use crate::{result::*, SpannedToken, Span};

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
    #[error("bitfield is missing representation scalar")]
    BitfieldMissingRep(Span),
    #[error("unknown container directive '{0}' @ {1}'")]
    UnknownContainerDirective(String, Span),

    #[error("unknown")]
    Unknown(#[from] crate::Error),
}
