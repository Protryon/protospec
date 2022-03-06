use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct UnaryExpression {
    pub op: UnaryOp,
    pub inner: Box<Expression>,
    pub span: Span,
}
impl_node!(UnaryExpression);

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug)]
pub enum UnaryOp {
    Negate,
    Not,
    BitNot,
}
