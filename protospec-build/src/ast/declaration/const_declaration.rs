use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct ConstDeclaration {
    pub name: Ident,
    pub type_: Type,
    pub value: Expression,
    pub span: Span,
}
impl_node!(ConstDeclaration);
