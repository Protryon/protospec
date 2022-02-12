use super::*;

#[derive(PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl_node!(Ident);
