use std::fmt;

use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};

use crate::{EndianScalarType, ScalarType};

#[derive(Debug)]
pub enum FieldRef {
    Ref,
    Name(String),
    ArrayAccess(usize),
    TupleAccess(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum Target {
    Direct,
    Buf(usize),
    Stream(usize),
}

impl Target {
    pub fn unwrap_buf(&self) -> usize {
        match self {
            Target::Buf(x) => *x,
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveType {
    Bool,
    F32,
    F64,
    Scalar(EndianScalarType),
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::F32 => write!(f, "f32"),
            PrimitiveType::F64 => write!(f, "f64"),
            PrimitiveType::Scalar(s) => write!(f, "{}", s),
        }
    }
}

impl PrimitiveType {
    pub fn size(&self) -> u64 {
        match self {
            PrimitiveType::Bool => 1,
            PrimitiveType::F32 => 4,
            PrimitiveType::F64 => 8,
            PrimitiveType::Scalar(s) => s.scalar.size(),
        }
    }
}

impl ToTokens for PrimitiveType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PrimitiveType::Bool => tokens.append(format_ident!("bool")),
            PrimitiveType::F32 => tokens.append(format_ident!("f32")),
            PrimitiveType::F64 => tokens.append(format_ident!("f64")),
            PrimitiveType::Scalar(s) => tokens.append(format_ident!("{}", &s.scalar.to_string())),
        }
    }
}

pub mod decode;
pub mod encode;
