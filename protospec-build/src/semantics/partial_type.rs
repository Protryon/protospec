use super::*;

#[derive(Clone, Debug)]
pub enum PartialType {
    Type(Type),
    Scalar(PartialScalarType),
    Array(Option<Box<PartialType>>),
    Any,
}

#[derive(Clone, Copy, Debug)]
pub enum PartialScalarType {
    Some(ScalarType),
    Defaults(ScalarType),
    None,
}

impl PartialType {
    pub fn assignable_from(&self, other: &Type) -> bool {
        match (self, other.resolved().as_ref()) {
            (t1, Type::Foreign(f2)) => f2.obj.assignable_to_partial(t1),
            (PartialType::Type(x), other) => x.assignable_from(other),
            (PartialType::Scalar(PartialScalarType::Some(x)), other) => Type::Scalar(*x).assignable_from(other),
            (PartialType::Scalar(PartialScalarType::Defaults(x)), other) => Type::Scalar(*x).assignable_from(other),
            (PartialType::Scalar(PartialScalarType::None), _) => true,
            (PartialType::Array(None), Type::Array(_)) => true,
            (PartialType::Array(Some(element)), Type::Array(array_type)) => {
                element.assignable_from(&array_type.element.type_.borrow())
            }
            (PartialType::Any, _) => true,
            _ => false,
        }
    }

    pub fn coercable_from(&self, other: &Type) -> bool {
        match (self, other.resolved().as_ref()) {
            (PartialType::Scalar(PartialScalarType::Some(scalar_type)), Type::Enum(e1))
                if e1.rep.can_implicit_cast_to(scalar_type) =>
            {
                true
            }
            (PartialType::Scalar(PartialScalarType::Defaults(scalar_type)), Type::Enum(e1))
                if e1.rep.can_implicit_cast_to(scalar_type) =>
            {
                true
            }
            (PartialType::Scalar(PartialScalarType::Some(x)), other) => other.can_coerce_to(&Type::Scalar(*x)),
            (PartialType::Scalar(PartialScalarType::Defaults(x)), other) => other.can_coerce_to(&Type::Scalar(*x)),
            (PartialType::Type(x), other) => other.can_coerce_to(x),
            (_, _) => false,
        }
    }

    pub fn into_type(&self) -> Option<Type> {
        Some(match self {
            PartialType::Type(t) => t.clone(),
            PartialType::Scalar(PartialScalarType::Some(s)) => Type::Scalar(*s),
            PartialType::Scalar(PartialScalarType::Defaults(s)) => Type::Scalar(*s),
            _ => return None,
        })
    }
}

impl fmt::Display for PartialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartialType::Type(t) => t.fmt(f),
            PartialType::Scalar(PartialScalarType::Some(s)) => s.fmt(f),
            PartialType::Scalar(PartialScalarType::Defaults(s)) => write!(f, "{}?", s),
            PartialType::Scalar(PartialScalarType::None) => write!(f, "integer"),
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
            Type::Scalar(x) => PartialType::Scalar(PartialScalarType::Some(x)),
            Type::Array(x) => {
                PartialType::Array(Some(Box::new(x.element.type_.borrow().clone().into())))
            }
            x => PartialType::Type(x),
        }
    }
}
