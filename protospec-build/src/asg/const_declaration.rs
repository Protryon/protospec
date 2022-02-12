use super::*;

#[derive(PartialEq, Debug)]
pub struct Const {
    pub name: String,
    pub type_: Type,
    pub span: Span,
    pub value: Expression,
}

impl AsgExpression for Const {
    fn get_type(&self) -> Option<Type> {
        Some(self.type_.clone())
    }
}
