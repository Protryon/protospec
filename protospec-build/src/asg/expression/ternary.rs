use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct TernaryExpression {
    pub condition: Box<Expression>,
    pub if_true: Box<Expression>,
    pub if_false: Box<Expression>,
    pub span: Span,
}

impl AsgExpression for TernaryExpression {
    fn get_type(&self) -> Option<Type> {
        self.if_true.get_type().or_else(|| self.if_false.get_type())
    }
}
