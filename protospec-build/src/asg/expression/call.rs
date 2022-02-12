use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct CallExpression {
    pub function: Arc<Function>,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

impl AsgExpression for CallExpression {
    fn get_type(&self) -> Option<Type> {
        Some(self.function.inner.return_type())
    }
}
