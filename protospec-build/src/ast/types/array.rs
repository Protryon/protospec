use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Array {
    pub element: Box<Field>,
    pub length: LengthConstraint,
    pub span: Span,
}
impl_node!(Array);

#[derive(Clone, Serialize, Deserialize)]
pub struct LengthConstraint {
    pub expandable: bool,
    pub inner: Option<Box<Expression>>,
    pub span: Span,
}
impl_node!(LengthConstraint);
