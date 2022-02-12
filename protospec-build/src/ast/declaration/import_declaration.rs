use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct ImportItem {
    pub name: Ident,
    pub alias: Option<Ident>,
    pub span: Span,
}
impl_node!(ImportItem);

#[derive(Clone, Serialize, Deserialize)]
pub struct ImportDeclaration {
    pub items: Vec<ImportItem>,
    pub from: Str,
    pub span: Span,
}
impl_node!(ImportDeclaration);
