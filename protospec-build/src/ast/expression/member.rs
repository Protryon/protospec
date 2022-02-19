use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct MemberExpression {
    pub target: Box<Expression>,
    pub member: Ident,
    pub span: Span,
}
impl_node!(MemberExpression);
