use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct MemberExpression {
    pub target: Box<Expression>,
    //todo: this might need to be more general in the future
    pub member: Arc<Const>,
    pub span: Span,
}

impl AsgExpression for MemberExpression {
    fn get_type(&self) -> Option<Type> {
        let type_ = self.target.get_type()?.resolved().into_owned();
        match &*type_.resolved() {
            Type::Bitfield(_) => {
                Some(Type::Bool)
            }
            _ => None,
        }
    }
}
