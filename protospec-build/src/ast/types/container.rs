use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub enum ContainerItem {
    Field(Ident, Field),
    Pad(Expression),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Container {
    pub length: Option<Box<Expression>>,
    pub items: Vec<ContainerItem>,
    pub flags: Vec<Ident>,
    pub span: Span,
}
impl_node!(Container);
