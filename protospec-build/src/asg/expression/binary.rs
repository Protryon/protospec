use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct BinaryExpression {
    pub op: BinaryOp,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub span: Span,
}

impl AsgExpression for BinaryExpression {
    fn get_type(&self) -> Option<Type> {
        use BinaryOp::*;
        match self.op {
            Lt | Gt | Lte | Gte | Eq | Ne | Or | And => Some(Type::Bool),
            _ => self.left.get_type().or(self.right.get_type()),
        }
    }
}
