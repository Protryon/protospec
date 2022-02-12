use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct UnaryExpression {
    pub op: UnaryOp,
    pub inner: Box<Expression>,
    pub span: Span,
}

impl AsgExpression for UnaryExpression {
    fn get_type(&self) -> Option<Type> {
        self.inner.get_type()
    }
}
