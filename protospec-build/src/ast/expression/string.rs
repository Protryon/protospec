use super::*;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Str {
    pub content: Vec<u8>,
    pub span: Span,
}

impl_node!(Str);
