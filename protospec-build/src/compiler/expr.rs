use std::{
    cmp::{Ordering, PartialOrd},
    ops::{Add, Div, Mul, Neg, Rem, Sub},
};

use proc_macro2::Literal;

use super::*;

#[derive(PartialEq)]
pub enum ConstValue {
    Int(ConstInt),
    Bool(bool),
    String(Vec<u8>),
    F32(f32),
    F64(f64),
}

impl PartialOrd for ConstValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (ConstValue::Int(i1), ConstValue::Int(i2)) => i1.partial_cmp(i2),
            (ConstValue::F32(i1), ConstValue::F32(i2)) => i1.partial_cmp(i2),
            (ConstValue::F64(i1), ConstValue::F64(i2)) => i1.partial_cmp(i2),
            _ => None,
        }
    }
}

impl Add for ConstValue {
    type Output = Option<Self>;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (ConstValue::Int(i1), ConstValue::Int(i2)) => Some(ConstValue::Int(i1.add(i2)?)),
            (ConstValue::F32(i1), ConstValue::F32(i2)) => Some(ConstValue::F32(i1.add(i2))),
            (ConstValue::F64(i1), ConstValue::F64(i2)) => Some(ConstValue::F64(i1.add(i2))),
            (ConstValue::String(mut i1), ConstValue::String(i2)) => {
                i1.extend(i2);
                Some(ConstValue::String(i1))
            }
            _ => None,
        }
    }
}

impl Sub for ConstValue {
    type Output = Option<Self>;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (ConstValue::Int(i1), ConstValue::Int(i2)) => Some(ConstValue::Int(i1.sub(i2)?)),
            (ConstValue::F32(i1), ConstValue::F32(i2)) => Some(ConstValue::F32(i1.sub(i2))),
            (ConstValue::F64(i1), ConstValue::F64(i2)) => Some(ConstValue::F64(i1.sub(i2))),
            _ => None,
        }
    }
}

impl Mul for ConstValue {
    type Output = Option<Self>;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (ConstValue::Int(i1), ConstValue::Int(i2)) => Some(ConstValue::Int(i1.mul(i2)?)),
            (ConstValue::F32(i1), ConstValue::F32(i2)) => Some(ConstValue::F32(i1.mul(i2))),
            (ConstValue::F64(i1), ConstValue::F64(i2)) => Some(ConstValue::F64(i1.mul(i2))),
            _ => None,
        }
    }
}

impl Div for ConstValue {
    type Output = Option<Self>;

    fn div(self, other: Self) -> Self::Output {
        match (self, other) {
            (ConstValue::Int(i1), ConstValue::Int(i2)) => Some(ConstValue::Int(i1.div(i2)?)),
            (ConstValue::F32(i1), ConstValue::F32(i2)) => Some(ConstValue::F32(i1.div(i2))),
            (ConstValue::F64(i1), ConstValue::F64(i2)) => Some(ConstValue::F64(i1.div(i2))),
            _ => None,
        }
    }
}

impl Rem for ConstValue {
    type Output = Option<Self>;

    fn rem(self, other: Self) -> Self::Output {
        match (self, other) {
            (ConstValue::Int(i1), ConstValue::Int(i2)) => Some(ConstValue::Int(i1.rem(i2)?)),
            (ConstValue::F32(i1), ConstValue::F32(i2)) => Some(ConstValue::F32(i1.rem(i2))),
            (ConstValue::F64(i1), ConstValue::F64(i2)) => Some(ConstValue::F64(i1.rem(i2))),
            _ => None,
        }
    }
}

impl Neg for ConstValue {
    type Output = Option<Self>;

    #[allow(unreachable_code)]
    fn neg(self) -> Self::Output {
        match self {
            ConstValue::Int(i1) => Some(ConstValue::Int(i1.neg()?)),
            ConstValue::F32(i1) => Some(ConstValue::F32(i1.neg())),
            ConstValue::F64(i1) => Some(ConstValue::F64(i1.neg())),
            _ => None,
        }
    }
}

impl ConstValue {
    pub fn emit(&self) -> TokenStream {
        use ConstInt::*;
        match self {
            ConstValue::Int(i) => match i {
                I8(x) => quote! { #x },
                I16(x) => quote! { #x },
                I32(x) => quote! { #x },
                I64(x) => quote! { #x },
                I128(x) => quote! { #x },
                U8(x) => quote! { #x },
                U16(x) => quote! { #x },
                U32(x) => quote! { #x },
                U64(x) => quote! { #x },
                U128(x) => quote! { #x },
            },
            ConstValue::Bool(b) => quote! { #b },
            ConstValue::String(c) => {
                let c = Literal::byte_string(&c[..]);
                quote! {
                    #c
                }
            }
            ConstValue::F32(s) => quote! { #s },
            ConstValue::F64(s) => quote! { #s },
        }
    }

    pub fn cast_to(&self, target: &Type) -> Option<Self> {
        match (self, target) {
            (ConstValue::Int(i1), Type::Scalar(s)) => Some(ConstValue::Int(i1.cast_to(s.scalar))),
            (ConstValue::F32(i1), Type::F32) => Some(ConstValue::F32(*i1 as f32)),
            (ConstValue::F64(i1), Type::F32) => Some(ConstValue::F32(*i1 as f32)),
            (ConstValue::F32(i1), Type::F64) => Some(ConstValue::F64(*i1 as f64)),
            (ConstValue::F64(i1), Type::F64) => Some(ConstValue::F64(*i1 as f64)),
            //todo: int <-> float casting
            _ => None,
        }
    }

    fn to_bool(&self) -> Option<bool> {
        match self {
            ConstValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn to_int(&self) -> Option<ConstInt> {
        match self {
            ConstValue::Int(b) => Some(*b),
            _ => None,
        }
    }
}

pub fn eval_const_expression(expr: &Expression) -> Option<ConstValue> {
    use Expression::*;
    match expr {
        Binary(c) => {
            let left = eval_const_expression(&c.left)?;
            let right = eval_const_expression(&c.right)?;
            Some(match c.op {
                BinaryOp::Lt => ConstValue::Bool(left < right),
                BinaryOp::Gt => ConstValue::Bool(left > right),
                BinaryOp::Lte => ConstValue::Bool(left <= right),
                BinaryOp::Gte => ConstValue::Bool(left >= right),
                BinaryOp::Eq => ConstValue::Bool(left == right),
                BinaryOp::Ne => ConstValue::Bool(left != right),
                BinaryOp::Or => ConstValue::Bool(left.to_bool()? || right.to_bool()?),
                BinaryOp::And => ConstValue::Bool(left.to_bool()? && right.to_bool()?),
                BinaryOp::BitOr => ConstValue::Int((left.to_int()? | right.to_int()?)?),
                BinaryOp::BitAnd => ConstValue::Int((left.to_int()? & right.to_int()?)?),
                BinaryOp::BitXor => ConstValue::Int((left.to_int()? ^ right.to_int()?)?),
                BinaryOp::Shr => ConstValue::Int((left.to_int()? >> right.to_int()?)?), // todo: cast signedness to ensure proper shr is used in rust
                BinaryOp::Shl => ConstValue::Int((left.to_int()? << right.to_int()?)?),
                BinaryOp::ShrSigned => ConstValue::Int((left.to_int()? >> right.to_int()?)?),
                BinaryOp::Add => ConstValue::Int((left.to_int()? + right.to_int()?)?),
                BinaryOp::Sub => ConstValue::Int((left.to_int()? - right.to_int()?)?),
                BinaryOp::Mul => ConstValue::Int((left.to_int()? * right.to_int()?)?),
                BinaryOp::Div => ConstValue::Int((left.to_int()? / right.to_int()?)?),
                BinaryOp::Mod => ConstValue::Int((left.to_int()? % right.to_int()?)?),
                BinaryOp::Elvis => {
                    unimplemented!("cannot use elvis operator (?:) in const context")
                }
            })
        }
        Member(c) => {
            let target = eval_const_expression(&c.target)?;
            let member = eval_const_expression(&c.member.value)?;
            match (target, member) {
                (ConstValue::Int(target), ConstValue::Int(member)) => {
                    // todo: find better way to get same-typed zero
                    Some(ConstValue::Bool((target & member)? != (member - member)?))
                }
                _ => unimplemented!("can not use member exprs on nonint targets"),
            }
        }
        Unary(c) => {
            let inner = eval_const_expression(&c.inner)?;
            Some(match c.op {
                UnaryOp::Negate => ConstValue::Int((-inner.to_int()?)?),
                UnaryOp::Not => ConstValue::Bool(!inner.to_bool()?),
                UnaryOp::BitNot => ConstValue::Int((!inner.to_int()?)?),
            })
        }
        Cast(c) => {
            let inner = eval_const_expression(&c.inner)?;
            inner.cast_to(&c.type_)
        }
        ArrayIndex(_) => {
            unimplemented!("can not use array indexing in const")
        }
        EnumAccess(c) => eval_const_expression(&c.variant.value),
        Int(c) => Some(ConstValue::Int(c.value)),
        ConstRef(c) => eval_const_expression(&c.value),
        InputRef(_) => {
            unimplemented!("cannot access input in constant");
        }
        FieldRef(_) => {
            unimplemented!("cannot access field in constant");
        }
        Str(c) => Some(ConstValue::String(c.content.clone())),
        Ternary(c) => {
            let condition = eval_const_expression(&c.condition)?;
            let if_true = eval_const_expression(&c.if_true)?;
            let if_false = eval_const_expression(&c.if_false)?;

            match condition {
                ConstValue::Bool(true) => Some(if_true),
                ConstValue::Bool(false) => Some(if_false),
                _ => None,
            }
        }
        Bool(c) => Some(ConstValue::Bool(*c)),
        Call(_) => {
            unimplemented!("cannot call ffi in constant");
        }
    }
}

pub fn emit_expression<F: Fn(&Arc<Field>) -> TokenStream>(
    expr: &Expression,
    ref_resolver: &F,
) -> TokenStream {
    use Expression::*;
    match expr {
        Binary(c) => {
            let left = emit_expression(&c.left, ref_resolver);
            let right = emit_expression(&c.right, ref_resolver);
            if c.op == BinaryOp::Elvis {
                quote! {
                    (#left).unwrap_or(#right)
                }
            } else {
                let op = match c.op {
                    BinaryOp::Lt => quote! { < },
                    BinaryOp::Gt => quote! { > },
                    BinaryOp::Lte => quote! { <= },
                    BinaryOp::Gte => quote! { >= },
                    BinaryOp::Eq => quote! { == },
                    BinaryOp::Ne => quote! { != },
                    BinaryOp::Or => quote! { || },
                    BinaryOp::And => quote! { && },
                    BinaryOp::BitOr => quote! { | },
                    BinaryOp::BitAnd => quote! { & },
                    BinaryOp::BitXor => quote! { ^ },
                    BinaryOp::Shr => quote! { >> },
                    BinaryOp::Shl => quote! { << },
                    BinaryOp::ShrSigned => quote! { >>> },
                    BinaryOp::Add => quote! { + },
                    BinaryOp::Sub => quote! { - },
                    BinaryOp::Mul => quote! { * },
                    BinaryOp::Div => quote! { / },
                    BinaryOp::Mod => quote! { % },
                    BinaryOp::Elvis => unimplemented!(),
                };
                quote! {
                    ((#left) #op (#right))
                }
            }
        }
        Member(c) => {
            let target = emit_expression(&c.target, ref_resolver);
            let get_member = format_ident!("{}", c.member.name.to_snake());
            quote! {
                #target.#get_member()
            }
        }
        Unary(c) => {
            let inner = emit_expression(&c.inner, ref_resolver);
            let op = match c.op {
                UnaryOp::Negate => quote! { - },
                UnaryOp::Not => quote! { ! },
                UnaryOp::BitNot => quote! { ! },
            };
            quote! {
                (#op #inner)
            }
        }
        Cast(c) => {
            let inner = emit_expression(&c.inner, ref_resolver);
            let target = emit_type_ref(&c.type_);
            match &*c.inner.get_type().unwrap().resolved() {
                Type::Enum(_) => {
                    quote! {
                        (#inner).to_repr() as #target
                    }
                }
                Type::Bitfield(_) => {
                    quote! {
                        (#inner).0 as #target
                    }
                }
                _ => {
                    quote! {
                        (#inner) as #target
                    }
                }
            }
        }
        ArrayIndex(c) => {
            let array = emit_expression(&c.array, ref_resolver);
            let index = emit_expression(&c.index, ref_resolver);

            quote! {
                (#array)[#index]
            }
        }
        EnumAccess(c) => {
            let enum_name = emit_ident(&c.enum_field.name);
            let enum_variant_name = emit_ident(&c.variant.name);
            quote! {
                #enum_name::#enum_variant_name
            }
        }
        Int(c) => {
            use ConstInt::*;
            match c.value {
                I8(x) => quote! { #x },
                I16(x) => quote! { #x },
                I32(x) => quote! { #x },
                I64(x) => quote! { #x },
                I128(x) => quote! { #x },
                U8(x) => quote! { #x },
                U16(x) => quote! { #x },
                U32(x) => quote! { #x },
                U64(x) => quote! { #x },
                U128(x) => quote! { #x },
            }
        }
        ConstRef(c) => {
            let c = format_ident!("{}", c.name);
            quote! {
                #c
            }
        }
        InputRef(c) => {
            let c = format_ident!("{}", c.name);
            quote! {
                #c
            }
        }
        FieldRef(c) => ref_resolver(c),
        Str(c) => {
            let c = Literal::byte_string(&c.content[..]);
            quote! {
                #c
            }
        }
        Ternary(c) => {
            let condition = emit_expression(&c.condition, ref_resolver);
            let if_true = emit_expression(&c.if_true, ref_resolver);
            let if_false = emit_expression(&c.if_false, ref_resolver);

            quote! {
                if #condition {
                    #if_true
                } else {
                    #if_false
                }
            }
        }
        Bool(c) => {
            quote! {
                #c
            }
        }
        Call(c) => {
            let mut arguments = vec![];
            for argument in &c.arguments {
                let expression = emit_expression(argument, ref_resolver);
                arguments.push(FFIArgumentValue {
                    type_: argument.get_type().expect("missing type in ffi argument"),
                    present: true,
                    value: expression,
                });
            }
            for argument in c.function.arguments[c.arguments.len()..].iter() {
                arguments.push(FFIArgumentValue {
                    type_: argument.type_.clone().unwrap_or(Type::Bool),
                    present: false,
                    value: quote! {},
                });
            }
            c.function.inner.call(&arguments[..])
        }
    }
}
