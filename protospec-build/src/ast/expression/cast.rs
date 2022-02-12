use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct CastExpression {
    pub inner: Box<Expression>,
    pub type_: Type,
    pub span: Span,
}
impl_node!(CastExpression);
