use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct CallExpression {
    pub function: Ident,
    pub arguments: Vec<Expression>,
    pub span: Span,
}
impl_node!(CallExpression);
