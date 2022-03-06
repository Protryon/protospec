use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct ArrayIndexExpression {
    pub array: Box<Expression>,
    pub index: Box<Expression>,
    pub span: Span,
}

impl AsgExpression for ArrayIndexExpression {
    fn get_type(&self) -> Option<Type> {
        let parent_type = self.array.get_type()?;
        match parent_type {
            Type::Array(parent_type) => Some(parent_type.element.as_ref().clone()),
            _ => None,
        }
    }
}
