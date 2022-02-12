use super::*;

mod scalar;
pub use scalar::*;

mod array;
pub use array::*;

mod container;
pub use container::*;

mod enum_;
pub use enum_::*;

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
    Scalar(ScalarType),
    Array(Array),
    F32,
    F64,
    Bool,
    Ref(TypeRef),
}
