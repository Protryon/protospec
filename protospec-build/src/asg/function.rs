use super::*;

pub struct Function {
    pub name: String,
    pub span: Span,
    pub inner: ForeignFunctionObj,
    pub arguments: Vec<FFIArgument>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.name, self.span)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        self.name == other.name
    }
}
