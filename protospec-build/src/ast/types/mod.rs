use super::*;

mod scalar;
pub use scalar::*;

mod array;
pub use array::*;

mod container;
pub use container::*;

mod enum_;
pub use enum_::*;

mod bitfield;
pub use bitfield::*;

mod type_ref;
pub use type_ref::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Type {
    pub raw_type: RawType,
    pub span: Span,
}
impl_node!(Type);

#[derive(Clone, Serialize, Deserialize)]
pub enum RawType {
    Container(Container),
    Enum(Enum),
    Bitfield(Bitfield),
    Scalar(EndianScalarType),
    Array(Array),
    F32,
    F64,
    Bool,
    Ref(TypeRef),
}

impl RawType {
    pub fn is_inlinable(&self) -> bool {
        !matches!(
            self,
            RawType::Container(_) | RawType::Enum(_) | RawType::Bitfield(_)
        )
    }
}
