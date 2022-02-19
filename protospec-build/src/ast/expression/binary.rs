use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct BinaryExpression {
    pub op: BinaryOp,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub span: Span,
}
impl_node!(BinaryExpression);

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug)]
pub enum BinaryOp {
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Ne,
    Or,
    And,
    BitOr,
    BitAnd,
    BitXor,
    Shr,
    Shl,
    ShrSigned,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Elvis,
}
