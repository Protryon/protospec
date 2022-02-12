use super::*;

pub struct Transform {
    pub name: String,
    pub span: Span,
    pub inner: ForeignTransformObj,
    pub arguments: Vec<FFIArgument>,
}

impl fmt::Debug for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.name, self.span)
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Transform) -> bool {
        self.name == other.name
    }
}
