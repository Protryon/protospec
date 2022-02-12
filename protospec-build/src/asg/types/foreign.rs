use super::*;

pub struct ForeignType {
    pub name: String,
    pub span: Span,
    pub obj: ForeignTypeObj,
}

impl fmt::Debug for ForeignType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.name, self.span)
    }
}

impl PartialEq for ForeignType {
    fn eq(&self, other: &ForeignType) -> bool {
        self.name == other.name
    }
}
