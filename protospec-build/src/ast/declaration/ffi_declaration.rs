use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct FfiDeclaration {
    pub name: Ident,
    pub ffi_type: FfiType,
    pub span: Span,
}
impl_node!(FfiDeclaration);

#[derive(Clone, Serialize, Deserialize)]
pub enum FfiType {
    Transform,
    Type,
    Function,
}
