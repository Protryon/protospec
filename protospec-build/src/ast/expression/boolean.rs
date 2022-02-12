use super::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Bool {
    pub value: bool,
    pub span: Span,
}

impl_node!(Bool);
