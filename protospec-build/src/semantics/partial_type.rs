use super::*;

#[derive(Clone)]
pub enum PartialType {
    Type(Type),
    Scalar(Option<ScalarType>),
    Array(Option<Box<PartialType>>),
    Any,
}

impl PartialType {
    pub fn assignable_from(&self, other: &Type) -> bool {
        match (self, other.resolved().as_ref()) {
            (t1, Type::Foreign(f2)) => f2.obj.assignable_to_partial(t1),
            (PartialType::Scalar(scalar_type), Type::Enum(e1)) => {
                if let Some(scalar_type) = scalar_type {
                    e1.rep.can_implicit_cast_to(scalar_type)
                } else {
                    true
                }
            }
            (PartialType::Type(x), other) => x.assignable_from(other),
            (PartialType::Scalar(x), Type::Scalar(y)) => {
                x.map(|x| y.can_implicit_cast_to(&x)).unwrap_or(true)
            }
            (PartialType::Array(None), Type::Array(_)) => true,
            (PartialType::Array(Some(element)), Type::Array(array_type)) => {
                element.assignable_from(&array_type.element.type_.borrow())
            }
            (PartialType::Any, _) => true,
            _ => false,
        }
    }
}

impl fmt::Display for PartialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartialType::Type(t) => t.fmt(f),
            PartialType::Scalar(Some(s)) => s.fmt(f),
            PartialType::Scalar(None) => write!(f, "integer"),
            PartialType::Array(None) => write!(f, "array"),
            PartialType::Array(Some(inner)) => write!(f, "{}[]", inner),
            PartialType::Any => write!(f, "any"),
        }
    }
}

impl Into<PartialType> for Type {
    fn into(self) -> PartialType {
        match self {
            Type::Ref(x) => x.target.type_.borrow().clone().into(),
            Type::Scalar(x) => PartialType::Scalar(Some(x)),
            Type::Array(x) => {
                PartialType::Array(Some(Box::new(x.element.type_.borrow().clone().into())))
            }
            x => PartialType::Type(x),
        }
    }
}
