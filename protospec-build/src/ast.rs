use crate::Span;
use serde::{Deserialize, Serialize};
use std::fmt;

pub trait Node {
    fn span(&self) -> &Span;
}

macro_rules! impl_node {
    ($name:ident) => {
        impl Node for $name {
            fn span(&self) -> &Span {
                &self.span
            }
        }
    };
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Declaration {
    Type(TypeDeclaration),
    Import(ImportDeclaration),
    Ffi(FfiDeclaration),
    Const(ConstDeclaration),
}

impl Node for Declaration {
    fn span(&self) -> &Span {
        match self {
            Declaration::Type(x) => x.span(),
            Declaration::Import(x) => x.span(),
            Declaration::Ffi(x) => x.span(),
            Declaration::Const(x) => x.span(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TypeArgument {
    pub name: Ident,
    pub type_: Type,
    pub default_value: Option<Expression>,
    pub span: Span,
}
impl_node!(TypeArgument);

#[derive(Clone, Serialize, Deserialize)]
pub struct TypeDeclaration {
    pub name: Ident,
    pub arguments: Vec<TypeArgument>,
    pub value: Field,
    pub span: Span,
}
impl_node!(TypeDeclaration);

#[derive(Clone, Serialize, Deserialize)]
pub struct ImportItem {
    pub name: Ident,
    pub alias: Option<Ident>,
    pub span: Span,
}
impl_node!(ImportItem);

#[derive(Clone, Serialize, Deserialize)]
pub struct ImportDeclaration {
    pub items: Vec<ImportItem>,
    pub from: Str,
    pub span: Span,
}
impl_node!(ImportDeclaration);

#[derive(Clone, Serialize, Deserialize)]
pub struct FfiDeclaration {
    pub name: Ident,
    pub ffi_type: FfiType,
    pub span: Span,
}
impl_node!(FfiDeclaration);

#[derive(Clone, Serialize, Deserialize)]
pub enum FfiType {
    Transform,
    Type,
    Function,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConstDeclaration {
    pub name: Ident,
    pub type_: Type,
    pub value: Expression,
    pub span: Span,
}
impl_node!(ConstDeclaration);

#[derive(Clone, Serialize, Deserialize)]
pub struct LengthConstraint {
    pub expandable: bool,
    pub inner: Option<Box<Expression>>,
    pub span: Span,
}
impl_node!(LengthConstraint);

#[derive(Clone, Serialize, Deserialize, PartialEq, Copy, Debug)]
pub enum ScalarType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
}

impl ScalarType {
    pub fn can_implicit_cast_to(&self, to: &ScalarType) -> bool {
        if self.is_signed() != to.is_signed() {
            return false;
        }
        if self.size() > to.size() {
            return false;
        }
        true
    }

    pub fn is_signed(&self) -> bool {
        match self {
            ScalarType::I8 => true,
            ScalarType::I16 => true,
            ScalarType::I32 => true,
            ScalarType::I64 => true,
            ScalarType::I128 => true,
            _ => false,
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            ScalarType::I8 | ScalarType::U8 => 1,
            ScalarType::I16 | ScalarType::U16 => 2,
            ScalarType::I32 | ScalarType::U32 => 4,
            ScalarType::I64 | ScalarType::U64 => 8,
            ScalarType::I128 | ScalarType::U128 => 16,
        }
    }
}

impl fmt::Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ScalarType::*;
        write!(
            f,
            "{}",
            match self {
                U8 => "u8",
                U16 => "u16",
                U32 => "u32",
                U64 => "u64",
                U128 => "u128",
                I8 => "i8",
                I16 => "i16",
                I32 => "i32",
                I64 => "i64",
                I128 => "i128",
            }
        )
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TypeCall {
    pub name: Ident,
    pub arguments: Vec<Expression>,
    pub span: Span,
}
impl_node!(TypeCall);

#[derive(Clone, Serialize, Deserialize)]
pub enum RawType {
    Container(Container),
    Enum(Enum),
    Scalar(ScalarType),
    Array(ArrayType),
    F32,
    F64,
    Bool,
    Ref(TypeCall),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Transform {
    pub name: Ident,
    pub arguments: Vec<Expression>,
    pub conditional: Option<Box<Expression>>,
    pub span: Span,
}
impl_node!(Transform);

#[derive(Clone, Serialize, Deserialize)]
pub struct Type {
    pub raw_type: RawType,
    pub span: Span,
}
impl_node!(Type);

#[derive(Clone, Serialize, Deserialize)]
pub struct Field {
    pub type_: Type,
    pub flags: Vec<Ident>,
    pub condition: Option<Box<Expression>>,
    pub transforms: Vec<Transform>,
    pub span: Span,
}
impl_node!(Field);

#[derive(Clone, Serialize, Deserialize)]
pub struct Container {
    pub length: Option<Box<Expression>>,
    pub items: Vec<(Ident, Field)>,
    pub span: Span,
}
impl_node!(Container);

#[derive(Clone, Serialize, Deserialize)]
pub struct ArrayType {
    pub element: Box<Field>,
    pub length: LengthConstraint,
    pub span: Span,
}
impl_node!(ArrayType);

#[derive(Clone, Serialize, Deserialize)]
pub struct Enum {
    pub rep: ScalarType,
    pub items: Vec<(Ident, Option<Box<Expression>>)>,
    pub span: Span,
}
impl_node!(Enum);

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum BinaryOp {
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Ne,
    Or,
    And,
    BitOr,
    BitAnd,
    BitXor,
    Shr,
    Shl,
    ShrSigned,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Elvis,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum UnaryOp {
    Negate,
    Not,
    BitNot,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BinaryExpression {
    pub op: BinaryOp,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub span: Span,
}
impl_node!(BinaryExpression);

#[derive(Clone, Serialize, Deserialize)]
pub struct UnaryExpression {
    pub op: UnaryOp,
    pub inner: Box<Expression>,
    pub span: Span,
}
impl_node!(UnaryExpression);

#[derive(Clone, Serialize, Deserialize)]
pub struct CastExpression {
    pub inner: Box<Expression>,
    pub type_: Type,
    pub span: Span,
}
impl_node!(CastExpression);

#[derive(Clone, Serialize, Deserialize)]
pub struct ArrayIndexExpression {
    pub array: Box<Expression>,
    pub index: Box<Expression>,
    pub span: Span,
}
impl_node!(ArrayIndexExpression);

#[derive(Clone, Serialize, Deserialize)]
pub struct EnumAccessExpression {
    pub name: Ident,
    pub variant: Ident,
    pub span: Span,
}
impl_node!(EnumAccessExpression);

#[derive(Clone, Serialize, Deserialize)]
pub struct TernaryExpression {
    pub condition: Box<Expression>,
    pub if_true: Box<Expression>,
    pub if_false: Box<Expression>,
    pub span: Span,
}
impl_node!(TernaryExpression);

#[derive(Clone, Serialize, Deserialize)]
pub struct CallExpression {
    pub function: Ident,
    pub arguments: Vec<Expression>,
    pub span: Span,
}
impl_node!(CallExpression);

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

#[derive(PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl_node!(Ident);

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Str {
    pub content: Vec<u8>,
    pub span: Span,
}

impl_node!(Str);

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Int {
    pub value: String,
    pub type_: Option<ScalarType>,
    pub span: Span,
}

impl_node!(Int);

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Bool {
    pub value: bool,
    pub span: Span,
}

impl_node!(Bool);
