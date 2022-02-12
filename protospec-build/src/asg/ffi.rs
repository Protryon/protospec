use super::*;

pub struct FFIArgument {
    pub name: String,
    pub type_: Option<Type>,
    pub optional: bool,
}

pub struct FFIArgumentValue {
    pub type_: Type,
    pub present: bool,
    pub value: TokenStream,
}
