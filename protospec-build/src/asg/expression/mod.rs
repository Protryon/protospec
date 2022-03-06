use super::*;

mod binary;
pub use binary::*;

mod unary;
pub use unary::*;

mod cast;
pub use cast::*;

mod array_index;
pub use array_index::*;

mod enum_access;
pub use enum_access::*;

mod ternary;
pub use ternary::*;

mod call;
pub use call::*;

mod const_int;
pub use const_int::*;

mod int;
pub use int::*;

mod member;
pub use member::*;

pub trait AsgExpression {
    fn get_type(&self) -> Option<Type>;
}

#[derive(PartialEq, Clone)]
pub enum Expression {
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Cast(CastExpression),
    ArrayIndex(ArrayIndexExpression),
    EnumAccess(EnumAccessExpression),
    Int(Int),
    ConstRef(Arc<Const>),
    InputRef(Arc<Input>),
    FieldRef(Arc<Field>),
    Str(ast::Str),
    Ternary(TernaryExpression),
    Bool(bool),
    Call(CallExpression),
    Member(MemberExpression),
}

impl AsgExpression for Expression {
    fn get_type(&self) -> Option<Type> {
        use Expression::*;
        match self {
            Binary(e) => e.get_type(),
            Unary(e) => e.get_type(),
            Cast(e) => e.get_type(),
            ArrayIndex(e) => e.get_type(),
            EnumAccess(e) => e.get_type(),
            Int(e) => Some(Type::Scalar(e.type_)),
            ConstRef(e) => e.get_type(),
            InputRef(e) => e.get_type(),
            FieldRef(e) => e.get_type(),
            Str(e) => Some(Type::Array(Box::new(ArrayType {
                element: Box::new(Type::Scalar(ScalarType::U8)),
                length: LengthConstraint {
                    expandable: true,
                    value: Some(Expression::Int(self::Int {
                        value: ConstInt::U64(e.content.len() as u64),
                        type_: ScalarType::U64,
                        span: e.span,
                    })),
                },
            }))),
            Ternary(e) => e.get_type(),
            Bool(_) => Some(Type::Bool),
            Call(ffi) => ffi.get_type(),
            Member(e) => e.get_type(),
        }
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary(arg0) => f.debug_tuple("Binary").field(arg0).finish(),
            Self::Unary(arg0) => f.debug_tuple("Unary").field(arg0).finish(),
            Self::Cast(arg0) => f.debug_tuple("Cast").field(arg0).finish(),
            Self::ArrayIndex(arg0) => f.debug_tuple("ArrayIndex").field(arg0).finish(),
            Self::EnumAccess(arg0) => f.debug_tuple("EnumAccess").field(arg0).finish(),
            Self::Int(arg0) => f.debug_tuple("Int").field(arg0).finish(),
            Self::ConstRef(arg0) => f.debug_tuple("ConstRef").field(arg0).finish(),
            Self::InputRef(arg0) => f.debug_tuple("InputRef").field(arg0).finish(),
            Self::FieldRef(arg0) => f.debug_tuple("FieldRef").field(&arg0.name).finish(),
            Self::Str(arg0) => f.debug_tuple("Str").field(arg0).finish(),
            Self::Ternary(arg0) => f.debug_tuple("Ternary").field(arg0).finish(),
            Self::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            Self::Call(arg0) => f.debug_tuple("Call").field(arg0).finish(),
            Self::Member(arg0) => f.debug_tuple("Member").field(arg0).finish(),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "..")
        // use Expression::*;
        // match self {
        //     Binary(e) => e.fmt(f),
        //     Unary(e) => e.fmt(f),
        //     Cast(e) => e.fmt(f),
        //     ArrayIndex(e) => e.fmt(f),
        //     Int(e) => e.fmt(f),
        //     ConstRef(e) => e.fmt(f),
        //     FieldRef(e) => e.fmt(f),
        //     Str(e) => e.fmt(f),
        //     Ternary(e) => e.fmt(f),
        // }
    }
}

impl From<Int> for Expression {
    fn from(from: Int) -> Self {
        Expression::Int(from)
    }
}

impl From<u64> for Expression {
    fn from(from: u64) -> Self {
        Expression::Int(from.into())
    }
}