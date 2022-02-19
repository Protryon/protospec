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

#[derive(PartialEq, Clone, Debug)]
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
                element: Arc::new(Field {
                    name: "$string".to_string(),
                    arguments: RefCell::new(vec![]),
                    span: e.span,
                    type_: RefCell::new(Type::Scalar(ScalarType::U8)),
                    condition: RefCell::new(None),
                    transforms: RefCell::new(vec![]),
                    toplevel: false,
                    is_auto: Cell::new(false),
                    is_maybe_cyclical: Cell::new(false),
                    is_pad: Cell::new(false),
                }),
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
