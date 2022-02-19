use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct EnumAccessExpression {
    pub enum_field: Arc<Field>,
    pub variant: Arc<Const>,
    pub span: Span,
}

impl AsgExpression for EnumAccessExpression {
    fn get_type(&self) -> Option<Type> {
        match &*self.enum_field.type_.borrow() {
            Type::Enum(e) => Some(Type::Scalar(e.rep)),
            Type::Bitfield(e) => Some(Type::Scalar(e.rep)),
            _ => None,
        }
    }
}
