use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct BitfieldType {
    pub name: String,
    pub rep: EndianScalarType,
    pub items: IndexMap<String, Arc<Const>>,
}
