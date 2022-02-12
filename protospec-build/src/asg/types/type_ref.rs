use super::*;

#[derive(Clone, Debug)]
pub struct TypeRef {
    pub target: Arc<Field>,
    pub arguments: Vec<Expression>,
}
