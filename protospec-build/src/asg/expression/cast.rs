use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct CastExpression {
    pub inner: Box<Expression>,
    pub type_: Type,
    pub span: Span,
}

impl AsgExpression for CastExpression {
    fn get_type(&self) -> Option<Type> {
        Some(self.type_.clone())
    }
}
