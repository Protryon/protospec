use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Transform {
    pub name: Ident,
    pub arguments: Vec<Expression>,
    pub conditional: Option<Box<Expression>>,
    pub span: Span,
}
impl_node!(Transform);
