use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Bitfield {
    pub rep: ScalarType,
    pub items: Vec<(Ident, Option<Box<Expression>>)>,
    pub span: Span,
}
impl_node!(Bitfield);
