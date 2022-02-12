use crate::PartialType;
use crate::{ast, AsgError, AsgResult, BinaryOp, ScalarType, Span, UnaryOp};
use indexmap::{IndexMap, IndexSet};
use proc_macro2::TokenStream;
use std::fmt;
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
};
use std::{
    cmp::Ordering,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub},
    sync::Arc,
};

pub type ForeignTypeObj = Box<dyn ForeignType + 'static>;

pub trait ForeignType {
    fn assignable_from(&self, type_: &Type) -> bool;

    fn assignable_to(&self, type_: &Type) -> bool;

    fn assignable_from_partial(&self, type_: &PartialType) -> bool {
        match type_ {
            PartialType::Type(t) => self.assignable_from(t),
            PartialType::Any => true,
            PartialType::Scalar(Some(scalar)) => self.assignable_from(&Type::Scalar(*scalar)),
            _ => false,
        }
    }

    fn assignable_to_partial(&self, type_: &PartialType) -> bool {
        match type_ {
            PartialType::Type(t) => self.assignable_to(t),
            PartialType::Any => true,
            PartialType::Scalar(Some(scalar)) => self.assignable_to(&Type::Scalar(*scalar)),
            _ => false,
        }
    }

    // generally an identifier to refer to the type
    fn type_ref(&self) -> TokenStream;

    /* output code should be a term expression that:
        1. the expression should read its input from an implicit identifier `reader` as a `&mut R` where R: Read
        2. can read an arbitrary number of bytes from `reader`
        3. returns a value of the foreign type
    */
    fn decoding_gen(
        &self,
        source: TokenStream,
        output_ref: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream;

    /* output code should be a single statement that:
        1. takes an expression `field_ref` as a reference to a value of the foreign type
        2. the statement should write its output to an implicit identifier `writer` as a `&mut W` where W: Write
    */
    fn encoding_gen(
        &self,
        target: TokenStream,
        field_ref: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream;

    fn arguments(&self) -> Vec<TypeArgument>;
}

pub type ForeignTransformObj = Box<dyn ForeignTransform + Send + Sync + 'static>;

pub trait ForeignTransform {
    fn decoding_gen(
        &self,
        input_stream: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream;

    fn encoding_gen(
        &self,
        input_stream: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream;

    fn arguments(&self) -> Vec<FFIArgument>;
}

pub type ForeignFunctionObj = Box<dyn ForeignFunction + Send + Sync + 'static>;

pub trait ForeignFunction {
    fn arguments(&self) -> Vec<FFIArgument>;

    fn return_type(&self) -> Type;

    fn call(&self, arguments: &[FFIArgumentValue]) -> TokenStream;
}

#[derive(Debug)]
pub struct Program {
    pub types: IndexMap<String, Arc<Field>>,
    pub consts: IndexMap<String, Arc<Const>>,
    pub transforms: IndexMap<String, Arc<Transform>>,
    pub functions: IndexMap<String, Arc<Function>>,
}

impl Program {
    pub fn scan_cycles(&self) {
        for (_, field) in &self.types {
            let mut interior_fields = IndexSet::new();
            field.get_indirect_contained_fields(&mut interior_fields);
            if interior_fields.contains(&field.name) {
                field.is_maybe_cyclical.set(true);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeArgument {
    pub name: String,
    pub type_: Type,
    pub default_value: Option<Expression>,
    pub can_resolve_auto: bool,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub arguments: RefCell<Vec<TypeArgument>>,
    pub span: Span,
    pub type_: RefCell<Type>,
    pub condition: RefCell<Option<Expression>>,
    pub transforms: RefCell<Vec<TypeTransform>>,
    pub toplevel: bool,
    pub is_auto: Cell<bool>,
    pub is_maybe_cyclical: Cell<bool>,
}

impl Field {
    fn get_indirect_contained_fields(&self, target: &mut IndexSet<String>) {
        let type_ = self.type_.borrow();
        match &*type_ {
            Type::Array(interior) => {
                if target.insert(interior.element.name.clone()) {
                    interior.element.get_indirect_contained_fields(target);
                }
            }
            Type::Container(interior) => {
                for (_, field) in &interior.items {
                    if target.insert(field.name.clone()) {
                        field.get_indirect_contained_fields(target);
                    }
                }
            }
            Type::Ref(call) => {
                if target.insert(call.target.name.clone()) {
                    call.target.get_indirect_contained_fields(target);
                }
            }
            _ => (),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.type_.borrow())?;
        if let Some(condition) = &*self.condition.borrow() {
            write!(f, " {{ {} }}", condition)?;
        }
        for transform in self.transforms.borrow().iter() {
            write!(f, " -> {}", transform.transform.name)?;
            if let Some(condition) = transform.condition.as_ref() {
                write!(f, " {{ {} }}", condition)?;
            }
        }
        Ok(())
    }
}

impl AsgExpression for Field {
    fn get_type(&self) -> Option<Type> {
        Some(self.type_.borrow().clone())
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Field) -> bool {
        self.name == other.name && self.type_.borrow().assignable_from(&other.type_.borrow())
    }
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

#[derive(PartialEq, Clone, Debug)]
pub struct ContainerType {
    pub length: Option<Expression>,
    pub items: IndexMap<String, Arc<Field>>,
    pub is_enum: Cell<bool>,
}

impl ContainerType {
    //todo: optimize this
    pub fn flatten_view<'a>(&'a self) -> impl Iterator<Item = (String, Arc<Field>)> + 'a {
        self.items
            .iter()
            .flat_map(|(name, field)| match &*field.type_.borrow() {
                Type::Container(x) => x.flatten_view().collect::<Vec<_>>(),
                _ => vec![(name.clone(), field.clone())],
            })
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ArrayType {
    pub element: Arc<Field>,
    pub length: LengthConstraint,
}

#[derive(PartialEq, Clone, Debug)]
pub struct EnumType {
    pub rep: ScalarType,
    pub items: IndexMap<String, Arc<Const>>,
}

pub struct NamedForeignType {
    pub name: String,
    pub span: Span,
    pub obj: ForeignTypeObj,
}

impl fmt::Debug for NamedForeignType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.name, self.span)
    }
}

impl PartialEq for NamedForeignType {
    fn eq(&self, other: &NamedForeignType) -> bool {
        self.name == other.name
    }
}

#[derive(Clone, Debug)]
pub struct TypeCall {
    pub target: Arc<Field>,
    pub arguments: Vec<Expression>,
}

#[derive(Clone, Debug)]
pub enum Type {
    Container(Box<ContainerType>),
    Enum(EnumType),
    Scalar(ScalarType),
    Array(Box<ArrayType>),
    Foreign(Arc<NamedForeignType>),
    F32,
    F64,
    Bool,
    Ref(TypeCall),
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
            (Type::Enum(e1), Type::Scalar(scalar_type))
                if scalar_type.can_implicit_cast_to(&e1.rep) =>
            {
                true
            }
            (Type::Scalar(scalar_type), Type::Enum(e1))
                if e1.rep.can_implicit_cast_to(scalar_type) =>
            {
                true
            }
            (Type::Scalar(s1), Type::Scalar(s2)) => s2.can_implicit_cast_to(s1),
            (Type::Array(a1), Type::Array(a2)) => a1 == a2,
            (Type::F32, Type::F32) => true,
            (Type::F64, Type::F32) => true,
            (Type::Bool, Type::Bool) => true,
            (_, _) => false,
        }
    }

    pub fn can_cast_to(&self, to: &Type) -> bool {
        if to.assignable_from(self) {
            return true;
        }
        match (self.resolved().as_ref(), to.resolved().as_ref()) {
            (Type::Scalar(_), Type::Scalar(_)) => true,
            (Type::F32, Type::F64) => true,
            (Type::F64, Type::F32) => true,
            (Type::F32, Type::Scalar(_)) => true,
            (Type::F64, Type::Scalar(_)) => true,
            _ => false,
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Type) -> bool {
        self.assignable_from(other)
    }
}

#[derive(Debug)]
pub struct TypeTransform {
    pub transform: Arc<Transform>,
    pub condition: Option<Expression>,
    pub arguments: Vec<Expression>,
}

#[derive(PartialEq, Debug)]
pub struct Const {
    pub name: String,
    pub type_: Type,
    pub span: Span,
    pub value: Expression,
}

impl AsgExpression for Const {
    fn get_type(&self) -> Option<Type> {
        Some(self.type_.clone())
    }
}

#[derive(PartialEq, Debug)]
pub struct Input {
    pub name: String,
    pub type_: Type,
}

impl AsgExpression for Input {
    fn get_type(&self) -> Option<Type> {
        Some(self.type_.clone())
    }
}

pub struct FFIArgument {
    pub name: String,
    pub type_: Option<Type>,
    pub optional: bool,
}

pub struct FFIArgumentValue {
    pub type_: Type,
    pub present: bool,
    pub value: TokenStream,
}

pub struct Transform {
    pub name: String,
    pub span: Span,
    pub inner: ForeignTransformObj,
    pub arguments: Vec<FFIArgument>,
}

impl fmt::Debug for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.name, self.span)
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Transform) -> bool {
        self.name == other.name
    }
}

pub struct Function {
    pub name: String,
    pub span: Span,
    pub inner: ForeignFunctionObj,
    pub arguments: Vec<FFIArgument>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.name, self.span)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        self.name == other.name
    }
}

pub trait AsgExpression {
    fn get_type(&self) -> Option<Type>;
}

#[derive(PartialEq, Clone, Debug)]
pub struct BinaryExpression {
    pub op: BinaryOp,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub span: Span,
}

impl AsgExpression for BinaryExpression {
    fn get_type(&self) -> Option<Type> {
        use BinaryOp::*;
        match self.op {
            Lt | Gt | Lte | Gte | Eq | Ne | Or | And => Some(Type::Bool),
            _ => self.left.get_type().or(self.right.get_type()),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct UnaryExpression {
    pub op: UnaryOp,
    pub inner: Box<Expression>,
    pub span: Span,
}

impl AsgExpression for UnaryExpression {
    fn get_type(&self) -> Option<Type> {
        self.inner.get_type()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct CastExpression {
    pub inner: Box<Expression>,
    pub type_: Type,
    pub span: Span,
}

impl AsgExpression for CastExpression {
    fn get_type(&self) -> Option<Type> {
        Some(self.type_.clone())
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ArrayIndexExpression {
    pub array: Box<Expression>,
    pub index: Box<Expression>,
    pub span: Span,
}

impl AsgExpression for ArrayIndexExpression {
    fn get_type(&self) -> Option<Type> {
        let parent_type = self.array.get_type()?;
        match parent_type {
            Type::Array(parent_type) => Some(parent_type.element.type_.borrow().clone()),
            _ => None,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct EnumAccessExpression {
    pub enum_field: Arc<Field>,
    pub variant: Arc<Const>,
    pub span: Span,
}

impl AsgExpression for EnumAccessExpression {
    fn get_type(&self) -> Option<Type> {
        match &*self.enum_field.type_.borrow() {
            Type::Enum(e) => Some(Type::Scalar(e.rep)),
            _ => None,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct TernaryExpression {
    pub condition: Box<Expression>,
    pub if_true: Box<Expression>,
    pub if_false: Box<Expression>,
    pub span: Span,
}

impl AsgExpression for TernaryExpression {
    fn get_type(&self) -> Option<Type> {
        self.if_true.get_type().or_else(|| self.if_false.get_type())
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct CallExpression {
    pub function: Arc<Function>,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

impl AsgExpression for CallExpression {
    fn get_type(&self) -> Option<Type> {
        Some(self.function.inner.return_type())
    }
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
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConstInt {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

macro_rules! const_int_biop {
    ($e1: expr, $e2: expr, $i1: ident, $i2: ident, $op: expr) => {
        match ($e1, $e2) {
            (ConstInt::I8($i1), ConstInt::I8($i2)) => $op,
            (ConstInt::I16($i1), ConstInt::I16($i2)) => $op,
            (ConstInt::I32($i1), ConstInt::I32($i2)) => $op,
            (ConstInt::I64($i1), ConstInt::I64($i2)) => $op,
            (ConstInt::I128($i1), ConstInt::I128($i2)) => $op,
            (ConstInt::U8($i1), ConstInt::U8($i2)) => $op,
            (ConstInt::U16($i1), ConstInt::U16($i2)) => $op,
            (ConstInt::U32($i1), ConstInt::U32($i2)) => $op,
            (ConstInt::U64($i1), ConstInt::U64($i2)) => $op,
            (ConstInt::U128($i1), ConstInt::U128($i2)) => $op,
            _ => None,
        }
    };
}

macro_rules! const_int_biop_map {
    ($e1: expr, $e2: expr, $i1: ident, $i2: ident, $op: expr) => {
        match ($e1, $e2) {
            (ConstInt::I8($i1), ConstInt::I8($i2)) => Some(ConstInt::I8($op)),
            (ConstInt::I16($i1), ConstInt::I16($i2)) => Some(ConstInt::I16($op)),
            (ConstInt::I32($i1), ConstInt::I32($i2)) => Some(ConstInt::I32($op)),
            (ConstInt::I64($i1), ConstInt::I64($i2)) => Some(ConstInt::I64($op)),
            (ConstInt::I128($i1), ConstInt::I128($i2)) => Some(ConstInt::I128($op)),
            (ConstInt::U8($i1), ConstInt::U8($i2)) => Some(ConstInt::U8($op)),
            (ConstInt::U16($i1), ConstInt::U16($i2)) => Some(ConstInt::U16($op)),
            (ConstInt::U32($i1), ConstInt::U32($i2)) => Some(ConstInt::U32($op)),
            (ConstInt::U64($i1), ConstInt::U64($i2)) => Some(ConstInt::U64($op)),
            (ConstInt::U128($i1), ConstInt::U128($i2)) => Some(ConstInt::U128($op)),
            _ => None,
        }
    };
}

macro_rules! const_int_map {
    ($e1: expr, $i1: ident, $sop: expr, $usop: expr) => {
        match $e1 {
            ConstInt::I8($i1) => Some(ConstInt::I8($sop)),
            ConstInt::I16($i1) => Some(ConstInt::I16($sop)),
            ConstInt::I32($i1) => Some(ConstInt::I32($sop)),
            ConstInt::I64($i1) => Some(ConstInt::I64($sop)),
            ConstInt::I128($i1) => Some(ConstInt::I128($sop)),
            ConstInt::U8($i1) => Some(ConstInt::U8($usop)),
            ConstInt::U16($i1) => Some(ConstInt::U16($usop)),
            ConstInt::U32($i1) => Some(ConstInt::U32($usop)),
            ConstInt::U64($i1) => Some(ConstInt::U64($usop)),
            ConstInt::U128($i1) => Some(ConstInt::U128($usop)),
        }
    };
}

macro_rules! const_int_op {
    ($e1: expr, $i1: ident, $op: expr) => {
        match $e1 {
            ConstInt::I8($i1) => $op,
            ConstInt::I16($i1) => $op,
            ConstInt::I32($i1) => $op,
            ConstInt::I64($i1) => $op,
            ConstInt::I128($i1) => $op,
            ConstInt::U8($i1) => $op,
            ConstInt::U16($i1) => $op,
            ConstInt::U32($i1) => $op,
            ConstInt::U64($i1) => $op,
            ConstInt::U128($i1) => $op,
        }
    };
}

impl PartialOrd for ConstInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        const_int_biop!(self, other, i1, i2, i1.partial_cmp(i2))
    }
}

impl BitOr for ConstInt {
    type Output = Option<Self>;

    fn bitor(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 | i2)
    }
}

impl BitAnd for ConstInt {
    type Output = Option<Self>;

    fn bitand(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 & i2)
    }
}

impl BitXor for ConstInt {
    type Output = Option<Self>;

    fn bitxor(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 ^ i2)
    }
}

impl Shr for ConstInt {
    type Output = Option<Self>;

    fn shr(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 >> i2)
    }
}

impl Shl for ConstInt {
    type Output = Option<Self>;

    fn shl(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 << i2)
    }
}

impl Add for ConstInt {
    type Output = Option<Self>;

    fn add(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 + i2)
    }
}

impl Sub for ConstInt {
    type Output = Option<Self>;

    fn sub(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 - i2)
    }
}

impl Mul for ConstInt {
    type Output = Option<Self>;

    fn mul(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 * i2)
    }
}

impl Div for ConstInt {
    type Output = Option<Self>;

    fn div(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 / i2)
    }
}

impl Rem for ConstInt {
    type Output = Option<Self>;

    fn rem(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 % i2)
    }
}

impl Neg for ConstInt {
    type Output = Option<Self>;

    #[allow(unreachable_code, unused_variables)]
    fn neg(self) -> Self::Output {
        const_int_map!(self, i1, -i1, unimplemented!("cannot neg unsigned value"))
    }
}

impl Not for ConstInt {
    type Output = Option<Self>;

    fn not(self) -> Self::Output {
        const_int_map!(self, i1, i1.not(), i1.not())
    }
}

impl ConstInt {
    pub fn cast_to(&self, target: ScalarType) -> Self {
        const_int_op!(
            self,
            i1,
            match target {
                ScalarType::I8 => ConstInt::I8(*i1 as i8),
                ScalarType::I16 => ConstInt::I16(*i1 as i16),
                ScalarType::I32 => ConstInt::I32(*i1 as i32),
                ScalarType::I64 => ConstInt::I64(*i1 as i64),
                ScalarType::I128 => ConstInt::I128(*i1 as i128),
                ScalarType::U8 => ConstInt::U8(*i1 as u8),
                ScalarType::U16 => ConstInt::U16(*i1 as u16),
                ScalarType::U32 => ConstInt::U32(*i1 as u32),
                ScalarType::U64 => ConstInt::U64(*i1 as u64),
                ScalarType::U128 => ConstInt::U128(*i1 as u128),
            }
        )
    }

    pub fn parse(scalar_type: ScalarType, value: &str, span: Span) -> AsgResult<ConstInt> {
        if value.starts_with("0x") {
            let value = &value[2..];
            return Ok(match scalar_type {
                ScalarType::I8 => ConstInt::I8(
                    i8::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::I16 => ConstInt::I16(
                    i16::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::I32 => ConstInt::I32(
                    i32::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::I64 => ConstInt::I64(
                    i64::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::I128 => ConstInt::I128(
                    i128::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::U8 => ConstInt::U8(
                    u8::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::U16 => ConstInt::U16(
                    u16::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::U32 => ConstInt::U32(
                    u32::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::U64 => ConstInt::U64(
                    u64::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::U128 => ConstInt::U128(
                    u128::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
            });
        }
        Ok(match scalar_type {
            ScalarType::I8 => ConstInt::I8(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::I16 => ConstInt::I16(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::I32 => ConstInt::I32(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::I64 => ConstInt::I64(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::I128 => ConstInt::I128(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::U8 => ConstInt::U8(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::U16 => ConstInt::U16(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::U32 => ConstInt::U32(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::U64 => ConstInt::U64(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::U128 => ConstInt::U128(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
        })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Int {
    pub value: ConstInt,
    pub type_: ScalarType,
    pub span: Span,
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

impl From<u64> for Int {
    fn from(from: u64) -> Self {
        Self {
            value: ConstInt::U64(from),
            type_: ScalarType::U64,
            span: Span::default(),
        }
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
