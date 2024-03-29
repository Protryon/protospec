use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct EnumType {
    pub name: String,
    pub rep: EndianScalarType,
    pub items: IndexMap<String, EnumValue>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum EnumValue {
    Value(Arc<Const>),
    Default,
}
