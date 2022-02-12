use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct TernaryExpression {
    pub condition: Box<Expression>,
    pub if_true: Box<Expression>,
    pub if_false: Box<Expression>,
    pub span: Span,
}
impl_node!(TernaryExpression);
