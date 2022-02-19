use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct BitfieldType {
    pub rep: ScalarType,
    pub items: IndexMap<String, Arc<Const>>,
}
