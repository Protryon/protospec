use std::fmt;

use crate::{result::*, Span};
use thiserror::Error;

pub type AsgResult<T> = StdResult<T, AsgError>;

#[derive(Error)]
pub enum AsgError {
    #[error("unresolved ffi import '{0}' @ {1}")]
    FfiMissing(String, Span),
    #[error("unresolved import '{0}' @ {1}")]
    ImportMissing(String, Span),
    #[error("unresolved import item '{0}' does not exist in module {1} @ {2}")]
    ImportUnresolved(String, String, Span),
    #[error("failed to parse import file '{0}' @ {1}: {2}")]
    ImportParse(String, Span, crate::parser::ParseError),
    #[error("type name already in use: '{0}' @ {1}, originally declared at {2}")]
    TypeRedefinition(String, Span, Span),
    #[error("transform name already in use: '{0}' @ {1}, originally declared at {2}")]
    TransformRedefinition(String, Span, Span),
    #[error("function name already in use: '{0}' @ {1}, originally declared at {2}")]
    FunctionRedefinition(String, Span, Span),
    #[error("const name already in use: '{0}' @ {1}, originally declared at {2}")]
    ConstRedefinition(String, Span, Span),
    #[error("const cannot declare complex type: '{0}' @ {1}")]
    ConstTypeDefinition(String, Span),
    #[error("cast cannot declare complex type @ {0}")]
    CastTypeDefinition(Span),
    #[error("complex types cannot be declared in this context @ {0}")]
    IllegalComplexTypeDefinition(Span),
    #[error("enum variant name already in use: '{0}' @ {1}, originally declared at {2}")]
    EnumVariantRedefinition(String, Span, Span),
    #[error("bitfield flag name already in use: '{0}' @ {1}, originally declared at {2}")]
    BitfieldFlagRedefinition(String, Span, Span),
    #[error("container field name already in use: '{0}' @ {1}, originally declared at {2}")]
    ContainerFieldRedefinition(String, Span, Span),
    #[error("referenced type '{0}' @ {1} not found")]
    UnresolvedType(String, Span),
    #[error("referenced variable '{0}' @ {1} not found")]
    UnresolvedVar(String, Span),
    #[error("referenced transform '{0}' @ {1} not found")]
    UnresolvedTransform(String, Span),
    #[error("referenced function '{0}' @ {1} not found")]
    UnresolvedFunction(String, Span),
    #[error("referenced transform '{0}' @ {1} cannot encode type {2}")]
    InvalidTransformInput(String, Span, String),
    #[error("referenced transform '{0}' @ {1} cannot cannot have condition because its target encoding type is not assignable to its input encoding type: {2} != {3}")]
    InvalidTransformCondition(String, Span, String, String),
    #[error("unexpected type got {0}, expected {1} @ {2}")]
    UnexpectedType(String, String, Span),
    #[error("illegal cast, cannot cast from {0} to {1} @ {2}")]
    IllegalCast(String, String, Span),
    #[error("reference enum variant for enum {0}, {1} @ {2} is not a valid variant")]
    UnresolvedEnumVariant(String, String, Span),
    #[error("could not infer type @ {0} (try adding more explicit types)")]
    UninferredType(Span),
    #[error("could not parse int {0} @ {1} @ {1}")]
    InvalidInt(String, Span),
    #[error("invalid number of arguments for ffi, expected {0} to {1} arguments, got {2} @ {3}")]
    InvalidFFIArgumentCount(usize, usize, usize, Span),
    #[error("invalid number of arguments for type, expected {0} to {1} arguments, got {2} @ {3}")]
    InvalidTypeArgumentCount(usize, usize, usize, Span),
    #[error("cannot have required arguments after optional arguments for type @ {0}")]
    InvalidTypeArgumentOrder(Span),
    #[error("invalid or unknown flag '{0}' @ {1}")]
    InvalidFlag(String, Span),
    #[error("illegal repitition of type -- outline the interior as a top level type declaration @ {0}")]
    InlineRepetition(Span),
    #[error("enums, bitfields, and enum containers must be top level @ {0}")]
    MustBeToplevel(Span),
    #[error("cannot have field after unconditional field in enum container @ {0}")]
    EnumContainerFieldAfterUnconditional(Span),
    #[error("cannot have pad in enum container @ {0}")]
    EnumContainerPad(Span),
    #[error("type `{0}` does not implement auto receiving @ {1}")]
    TypeNotAutoCompatible(String, Span),
    #[error("referenced bitfield member `{0}` does not exist @ {1}")]
    BitfieldMemberUndefined(String, Span),
    #[error("unknown: {0}")]
    Unknown(#[from] crate::Error),
}

impl fmt::Debug for AsgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
