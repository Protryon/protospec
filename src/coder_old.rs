use crate::{asg::*, ScalarType};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use std::sync::Arc;

#[derive(Debug)]
pub enum Instruction {
    CodeRef(Arc<Field>, Vec<Expression>),
    CodeEnum(Arc<Field>),
    CodePrimitive(PrimitiveType),
    CodePrimitiveArray(PrimitiveType, Expression),
    CodeField(Arc<Field>, Context),
    CodeContainer(Arc<Field>, Vec<(Arc<Field>, Context)>),
    Bounded(Context, Expression),
    Repeat(Context, Expression),
    RepeatUntilTerminator(Context, Option<Expression>),
    If(Expression, Context),
    Transform(Arc<Transform>, Vec<Expression>, Context),
    TransformIf(Expression, Arc<Transform>, Vec<Expression>, Context),
}

#[derive(Debug)]
pub struct Context {
    pub instructions: Vec<Instruction>,
}

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveType {
    Bool,
    F32,
    F64,
    Scalar(ScalarType),
}

impl PrimitiveType {
    pub fn size(&self) -> u64 {
        match self {
            PrimitiveType::Bool => 1,
            PrimitiveType::F32 => 4,
            PrimitiveType::F64 => 8,
            PrimitiveType::Scalar(s) => s.size(),
        }
    }
}

impl ToTokens for PrimitiveType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PrimitiveType::Bool => tokens.append(format_ident!("bool")),
            PrimitiveType::F32 => tokens.append(format_ident!("f32")),
            PrimitiveType::F64 => tokens.append(format_ident!("f64")),
            PrimitiveType::Scalar(s) => tokens.append(format_ident!("{}", &s.to_string())),
        }
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            instructions: vec![],
        }
    }

    pub fn encode_field(&mut self, field: &Arc<Field>) {
        let mut inner = Context::new();
        inner.encode_complex_type(field);
        for transform in field.transforms.borrow().iter() {
            let mut new_inner = Context::new();
            if let Some(condition) = &transform.condition {
                new_inner.instructions.push(Instruction::TransformIf(
                    condition.clone(),
                    transform.transform.clone(),
                    transform.arguments.clone(),
                    inner,
                ));
                inner = new_inner;
            } else {
                new_inner.instructions.push(Instruction::Transform(
                    transform.transform.clone(),
                    transform.arguments.clone(),
                    inner,
                ));
                inner = new_inner;
            }
        }
        if let Some(condition) = field.condition.borrow().as_ref() {
            inner = Context {
                instructions: vec![Instruction::If(condition.clone(), inner)],
            };
        }
        self.instructions
            .push(Instruction::CodeField(field.clone(), inner));
    }

    pub fn encode_complex_type(&mut self, field: &Arc<Field>) {
        match &field.type_ {
            Type::Container(c) => {
                if let Some(expr) = &c.length {
                    let mut inner = Context::new();
                    let mut fields = vec![];
                    for (_name, field) in c.items.iter() {
                        let mut field_inner = Context::new();
                        field_inner.encode_field(field);
                        fields.push((field.clone(), field_inner));
                    }
                    inner
                        .instructions
                        .push(Instruction::CodeContainer(field.clone(), fields));
                    self.instructions
                        .push(Instruction::Bounded(inner, expr.clone()));
                } else {
                    let mut fields = vec![];
                    for (_name, field) in c.items.iter() {
                        let mut field_inner = Context::new();
                        field_inner.encode_field(field);
                        fields.push((field.clone(), field_inner));
                    }
                    self.instructions
                        .push(Instruction::CodeContainer(field.clone(), fields));
                }
            },
            Type::Enum(_) => {
                self.instructions.push(Instruction::CodeEnum(field.clone()));
            },
            Type::Foreign(_) => {
                self.instructions
                    .push(Instruction::CodeRef(field.clone(), vec![]));
            },
            x => {
                self.encode_type(x);
            },
        }
    }

    pub fn encode_type(&mut self, type_: &Type) {
        match type_ {
            Type::Container(_) | Type::Enum(_) | Type::Foreign(_) => unimplemented!(),
            Type::Array(c) => {
                let mut inner = Context::new();
                inner.encode_field(&c.element);
                if c.length.expandable {
                    let terminator = c.length.value.as_ref().cloned();
                    //todo: create smart-terms for terminators
                    self.instructions
                        .push(Instruction::RepeatUntilTerminator(inner, terminator));
                } else {
                    let len = c.length.value.as_ref().cloned().unwrap();

                    if c.element.condition.borrow().is_none()
                        && c.element.transforms.borrow().len() == 0
                    {
                        match &c.element.type_ {
                            // todo: const-length type optimizations for container/array/foreign
                            Type::Container(_)
                            | Type::Array(_)
                            | Type::Foreign(_)
                            | Type::Ref(_) => (),
                            Type::Enum(x) => {
                                self.instructions.push(Instruction::CodePrimitiveArray(
                                    PrimitiveType::Scalar(x.rep),
                                    len,
                                ));
                                return;
                            }
                            Type::Scalar(x) => {
                                self.instructions.push(Instruction::CodePrimitiveArray(
                                    PrimitiveType::Scalar(*x),
                                    len,
                                ));
                                return;
                            }
                            Type::F32 => {
                                self.instructions
                                    .push(Instruction::CodePrimitiveArray(PrimitiveType::F32, len));
                                return;
                            }
                            Type::F64 => {
                                self.instructions
                                    .push(Instruction::CodePrimitiveArray(PrimitiveType::F64, len));
                                return;
                            }
                            Type::Bool => {
                                self.instructions.push(Instruction::CodePrimitiveArray(
                                    PrimitiveType::Bool,
                                    len,
                                ));
                                return;
                            }
                        }
                    }
                    self.instructions.push(Instruction::Repeat(inner, len));
                }
            }
            Type::Scalar(c) => {
                self.instructions
                    .push(Instruction::CodePrimitive(PrimitiveType::Scalar(*c)));
            }
            Type::F32 => {
                self.instructions
                    .push(Instruction::CodePrimitive(PrimitiveType::F32));
            }
            Type::F64 => {
                self.instructions
                    .push(Instruction::CodePrimitive(PrimitiveType::F64));
            }
            Type::Bool => {
                self.instructions
                    .push(Instruction::CodePrimitive(PrimitiveType::Bool));
            }
            Type::Ref(field) => {
                self.instructions.push(Instruction::CodeRef(
                    field.target.clone(),
                    field.arguments.clone(),
                ));
            }
        }
    }
}
