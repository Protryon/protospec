use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct TypeArgument {
    pub name: Ident,
    pub type_: Type,
    pub default_value: Option<Expression>,
    pub span: Span,
}
impl_node!(TypeArgument);

#[derive(Clone, Serialize, Deserialize)]
pub struct TypeDeclaration {
    pub name: Ident,
    pub arguments: Vec<TypeArgument>,
    pub value: Field,
    pub span: Span,
}
impl_node!(TypeDeclaration);
