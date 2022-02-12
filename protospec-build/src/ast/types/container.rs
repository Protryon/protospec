use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Container {
    pub length: Option<Box<Expression>>,
    pub items: Vec<(Ident, Field)>,
    pub flags: Vec<Ident>,
    pub span: Span,
}
impl_node!(Container);
