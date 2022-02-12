use crate::Span;
use serde::{Deserialize, Serialize};
use std::fmt;

pub trait Node {
    fn span(&self) -> &Span;
}

macro_rules! impl_node {
    ($name:ident) => {
        impl Node for $name {
            fn span(&self) -> &Span {
                &self.span
            }
        }
    };
}

mod program;
pub use program::*;

mod declaration;
pub use declaration::*;

mod types;
pub use types::*;

mod transform;
pub use transform::*;

mod field;
pub use field::*;

mod expression;
pub use expression::*;
