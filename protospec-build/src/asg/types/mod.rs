use super::*;

mod array;
pub use array::*;

mod container;
pub use container::*;

mod enum_;
pub use enum_::*;

mod bitfield;
pub use bitfield::*;

mod foreign;
pub use foreign::*;

mod type_ref;
pub use type_ref::*;

#[derive(Clone, Debug)]
pub enum Type {
    Container(Box<ContainerType>),
    Enum(EnumType),
    Bitfield(BitfieldType),
    Scalar(ScalarType),
    Array(Box<ArrayType>),
    Foreign(Arc<ForeignType>),
    F32,
    F64,
    Bool,
    Ref(TypeRef),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Container(c) => {
                write!(f, "container ")?;
                if let Some(length) = c.length.as_ref() {
                    write!(f, "[{}] ", length)?;
                }
                write!(f, "{{\n")?;
                for (name, field) in c.items.iter() {
                    write!(f, "  {}: {}", name, field.type_.borrow())?;
                }
                write!(f, "\n}}\n")
            }
            Type::Enum(c) => {
                write!(f, "enum {} {{\n", c.rep)?;
                for (name, cons) in c.items.iter() {
                    write!(f, "  {} = {}", name, cons.value)?;
                }
                write!(f, "\n}}\n")
            }
            Type::Bitfield(c) => {
                write!(f, "bitfield {} {{\n", c.rep)?;
                for (name, cons) in c.items.iter() {
                    write!(f, "  {} = {}", name, cons.value)?;
                }
                write!(f, "\n}}\n")
            }
            Type::Scalar(c) => c.fmt(f),
            Type::Array(c) => {
                write!(f, "{}[{}]", c.element, c.length)
            }
            Type::Foreign(c) => {
                write!(f, "{}", c.name)
            }
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Ref(field) => {
                write!(f, "{}", field.target.name)?;
                if field.arguments.len() > 0 {
                    write!(f, "(")?;
                    for argument in field.arguments.iter() {
                        argument.fmt(f)?;
                        write!(f, ",")?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
        }
    }
}

impl Type {
    pub fn resolved(&self) -> Cow<'_, Type> {
        match self {
            Type::Ref(x) => Cow::Owned(x.target.type_.borrow().resolved().into_owned()),
            x => Cow::Borrowed(x),
        }
    }

    pub fn assignable_from(&self, other: &Type) -> bool {
        match (self.resolved().as_ref(), other.resolved().as_ref()) {
            (Type::Ref(field1), Type::Ref(field2)) => field1
                .target
                .type_
                .borrow()
                .assignable_from(&field2.target.type_.borrow()),
            (Type::Ref(field), x) => field.target.type_.borrow().assignable_from(x),
            (x, Type::Ref(field)) => x.assignable_from(&field.target.type_.borrow()),
            (t1, Type::Foreign(f2)) => f2.obj.assignable_to(t1),
            (Type::Foreign(f1), t2) => f1.obj.assignable_from(t2),
            (Type::Container(c1), Type::Container(c2)) => c1 == c2,
            (Type::Enum(e1), Type::Enum(e2)) => e1 == e2,
            (Type::Bitfield(e1), Type::Bitfield(e2)) => e1 == e2,
            (Type::Scalar(s1), Type::Scalar(s2)) => s2 == s1,
            (Type::Array(a1), Type::Array(a2)) => a1 == a2,
            (Type::F32, Type::F32) => true,
            (Type::F64, Type::F32) => true,
            (Type::Bool, Type::Bool) => true,
            (_, _) => false,
        }
    }

    pub fn can_coerce_to(&self, other: &Type) -> bool {
        match (self.resolved().as_ref(), other.resolved().as_ref()) {
            (Type::Enum(e1), Type::Scalar(scalar_type))
                if e1.rep.can_implicit_cast_to(scalar_type) =>
            {
                true
            }
            (Type::Bitfield(e1), Type::Scalar(scalar_type))
                if e1.rep.can_implicit_cast_to(scalar_type) =>
            {
                true
            }
            (Type::Scalar(scalar_type), Type::Enum(e1))
                if scalar_type.can_implicit_cast_to(&e1.rep) =>
            {
                true
            }
            (Type::Scalar(scalar_type), Type::Bitfield(e1))
                if scalar_type.can_implicit_cast_to(&e1.rep) =>
            {
                true
            }
            (Type::Scalar(s1), Type::Scalar(s2)) => s1.can_implicit_cast_to(s2),
            (_, _) => false,
        }
    }

    pub fn can_cast_to(&self, to: &Type) -> bool {
        if to.assignable_from(self) || to.can_coerce_to(self) {
            return true;
        }
        match (self.resolved().as_ref(), to.resolved().as_ref()) {
            (Type::Scalar(_), Type::Scalar(_)) => true,
            (Type::F32, Type::F64) => true,
            (Type::F64, Type::F32) => true,
            (Type::F32, Type::Scalar(_)) => true,
            (Type::F64, Type::Scalar(_)) => true,
            (Type::Scalar(_), Type::F32) => true,
            (Type::Scalar(_), Type::F64) => true,
            _ => false,
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Type) -> bool {
        self.assignable_from(other)
    }
}
