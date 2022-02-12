use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct TypeRef {
    pub name: Ident,
    pub arguments: Vec<Expression>,
    pub span: Span,
}
impl_node!(TypeRef);
