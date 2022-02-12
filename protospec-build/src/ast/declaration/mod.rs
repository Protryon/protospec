use super::*;

mod type_declaration;
pub use type_declaration::*;

mod import_declaration;
pub use import_declaration::*;

mod ffi_declaration;
pub use ffi_declaration::*;

mod const_declaration;
pub use const_declaration::*;

#[derive(Clone, Serialize, Deserialize)]
pub enum Declaration {
    Type(TypeDeclaration),
    Import(ImportDeclaration),
    Ffi(FfiDeclaration),
    Const(ConstDeclaration),
}

impl Node for Declaration {
    fn span(&self) -> &Span {
        match self {
            Declaration::Type(x) => x.span(),
            Declaration::Import(x) => x.span(),
            Declaration::Ffi(x) => x.span(),
            Declaration::Const(x) => x.span(),
        }
    }
}
