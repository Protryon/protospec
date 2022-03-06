use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};

use crate::ScalarType;

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
    Scalar(ScalarType),
}

impl PrimitiveType {
    pub fn size(&self) -> u64 {
        match self {
            PrimitiveType::Bool => 1,
            PrimitiveType::F32 => 4,
            PrimitiveType::F64 => 8,
            PrimitiveType::Scalar(s) => s.size(),
        }
    }
}

impl ToString for PrimitiveType {
    fn to_string(&self) -> String {
        match self {
            PrimitiveType::Bool => "bool".to_string(),
            PrimitiveType::F32 => "f32".to_string(),
            PrimitiveType::F64 => "f64".to_string(),
            PrimitiveType::Scalar(s) => s.to_string(),
        }
    }
}

impl ToTokens for PrimitiveType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PrimitiveType::Bool => tokens.append(format_ident!("bool")),
            PrimitiveType::F32 => tokens.append(format_ident!("f32")),
            PrimitiveType::F64 => tokens.append(format_ident!("f64")),
            PrimitiveType::Scalar(s) => tokens.append(format_ident!("{}", &s.to_string())),
        }
    }
}

pub mod decode;
pub mod encode;
