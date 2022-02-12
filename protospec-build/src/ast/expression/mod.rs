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

mod ident;
pub use ident::*;

mod string;
pub use string::*;

mod int;
pub use int::*;

mod boolean;
pub use boolean::*;

#[derive(Clone, Serialize, Deserialize)]
pub enum Expression {
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Cast(CastExpression),
    ArrayIndex(ArrayIndexExpression),
    EnumAccess(EnumAccessExpression),
    Int(Int),
    Ref(Ident),
    Str(Str),
    Ternary(TernaryExpression),
    Bool(Bool),
    Call(CallExpression),
}

impl Node for Expression {
    fn span(&self) -> &Span {
        match self {
            Expression::Binary(x) => x.span(),
            Expression::Unary(x) => x.span(),
            Expression::Cast(x) => x.span(),
            Expression::ArrayIndex(x) => x.span(),
            Expression::EnumAccess(x) => x.span(),
            Expression::Int(x) => x.span(),
            Expression::Ref(x) => x.span(),
            Expression::Str(x) => x.span(),
            Expression::Ternary(x) => x.span(),
            Expression::Bool(x) => x.span(),
            Expression::Call(x) => x.span(),
        }
    }
}
