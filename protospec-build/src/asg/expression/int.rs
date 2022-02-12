use super::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Int {
    pub value: ConstInt,
    pub type_: ScalarType,
    pub span: Span,
}

impl From<u64> for Int {
    fn from(from: u64) -> Self {
        Self {
            value: ConstInt::U64(from),
            type_: ScalarType::U64,
            span: Span::default(),
        }
    }
}
