use super::*;

#[derive(PartialEq, Debug)]
pub struct Input {
    pub name: String,
    pub type_: Type,
}

impl AsgExpression for Input {
    fn get_type(&self) -> Option<Type> {
        Some(self.type_.clone())
    }
}
