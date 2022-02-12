use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct EnumType {
    pub rep: ScalarType,
    pub items: IndexMap<String, Arc<Const>>,
}
