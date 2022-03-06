use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct ArrayType {
    pub element: Box<Type>,
    pub length: LengthConstraint,
}

#[derive(PartialEq, Clone, Debug)]
pub struct LengthConstraint {
    pub expandable: bool,
    pub value: Option<Expression>,
}

impl fmt::Display for LengthConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.expandable {
            write!(f, "..")?;
        }
        if let Some(value) = self.value.as_ref() {
            value.fmt(f)?;
        }
        write!(f, "")
    }
}
