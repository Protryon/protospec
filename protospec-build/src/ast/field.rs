use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Field {
    pub type_: Type,
    pub flags: Vec<Ident>,
    pub calculated: Option<Box<Expression>>,
    pub condition: Option<Box<Expression>>,
    pub transforms: Vec<Transform>,
    pub span: Span,
}
impl_node!(Field);
