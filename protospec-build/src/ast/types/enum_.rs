use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub enum EnumValue {
    Expression(Box<Expression>),
    Default,
    None,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Enum {
    pub rep: EndianScalarType,
    pub items: Vec<(Ident, EnumValue)>,
    pub span: Span,
}
impl_node!(Enum);
