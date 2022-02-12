use super::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Int {
    pub value: String,
    pub type_: Option<ScalarType>,
    pub span: Span,
}

impl_node!(Int);
