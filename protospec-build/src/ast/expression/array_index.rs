use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct ArrayIndexExpression {
    pub array: Box<Expression>,
    pub index: Box<Expression>,
    pub span: Span,
}
impl_node!(ArrayIndexExpression);
