use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct EnumAccessExpression {
    pub enum_field: Arc<Field>,
    pub variant: Arc<Const>,
    pub span: Span,
}

impl AsgExpression for EnumAccessExpression {
    fn get_type(&self) -> Option<Type> {
        Some(self.enum_field.type_.borrow().clone())
    }
}
