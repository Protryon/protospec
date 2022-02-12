use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct EnumAccessExpression {
    pub name: Ident,
    pub variant: Ident,
    pub span: Span,
}
impl_node!(EnumAccessExpression);
