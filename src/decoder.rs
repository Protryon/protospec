use crate::{ScalarType, asg::*};
use std::sync::Arc;
use quote::{ ToTokens, TokenStreamExt };
use proc_macro2::TokenStream;

impl CoderContext {
    pub fn new() -> CoderContext {
        CoderContext {
            instructions: vec![],
        }
    }

    pub fn decode_field(&mut self, field: &Arc<Field>) {
        let mut inner = CoderContext {
            instructions: vec![],
        };
        inner.decode_complex_type(field);
        for transform in field.transforms.borrow().iter() {
            let mut new_inner = CoderContext::new();
            if let Some(condition) = &transform.condition {
                new_inner.instructions.push(Instruction::TransformIf(condition.clone(), transform.transform.clone(), inner));
                inner = new_inner;
            } else {
                new_inner.instructions.push(Instruction::Transform(transform.transform.clone(), inner));
                inner = new_inner;
            }
        }
        if let Some(condition) = field.condition.borrow().as_ref() {
            inner = CoderContext {
                instructions: vec![Instruction::If(condition.clone(), inner)],
            };
        }
        self.instructions.push(Instruction::WriteToField(field.clone(), inner));
    }

    pub fn decode_complex_type(&mut self, field: &Arc<Field>) {
        match &field.type_ {
            Type::Container(c) => {
                if let Some(expr) = &c.length {
                    let mut inner = CoderContext {
                        instructions: vec![],
                    };
                    for (_name, field) in c.items.iter() {
                        inner.decode_field(field);
                    }
                    inner.instructions.push(Instruction::ConstructContainer(field.clone()));
                    self.instructions.push(Instruction::Bounded(inner, expr.clone()));
                } else {
                    for (_name, field) in c.items.iter() {
                        self.decode_field(field);
                    }
                    self.instructions.push(Instruction::ConstructContainer(field.clone()));
                }
            },
            x => {
                self.decode_type(x);
            }
        }
    }

    pub fn decode_type(&mut self, type_: &Type) {
        match type_ {
            Type::Container(c) => unimplemented!(),
            Type::Array(c) => {
                let mut inner = CoderContext {
                    instructions: vec![],
                };
                inner.decode_field(&c.element);
                if c.length.expandable {
                    let terminator = c.length.value.as_ref().cloned();
                    //todo: create smart-terms for terminators
                    self.instructions.push(Instruction::RepeatUntilTerminator(inner, terminator));
                } else {
                    let len = c.length.value.as_ref().cloned().unwrap();

                    if c.element.condition.borrow().is_none() && c.element.transforms.borrow().len() == 0 {
                        match &c.element.type_ {
                            // todo: const-length type optimizations for container/array/foreign
                            Type::Container(_) | Type::Array(_) | Type::Foreign(_) | Type::Ref(_) => (),
                            Type::Enum(x) => {
                                self.instructions.push(Instruction::ReadPrimitiveArray(PrimitiveType::Scalar(x.rep), len));
                                return;
                            },
                            Type::Scalar(x) => {
                                self.instructions.push(Instruction::ReadPrimitiveArray(PrimitiveType::Scalar(*x), len));
                                return;
                            },
                            Type::F32 => {
                                self.instructions.push(Instruction::ReadPrimitiveArray(PrimitiveType::F32, len));
                                return;
                            },
                            Type::F64 => {
                                self.instructions.push(Instruction::ReadPrimitiveArray(PrimitiveType::F64, len));
                                return;
                            },
                            Type::Bool => {
                                self.instructions.push(Instruction::ReadPrimitiveArray(PrimitiveType::Bool, len));
                                return;
                            },
                        }
                    }
                    self.instructions.push(Instruction::Repeat(inner, len));   
                }
                
            },
            Type::Foreign(c) => {
                self.instructions.push(Instruction::ReadForeign(c.clone()));
            },
            Type::Enum(c) => {
                self.instructions.push(Instruction::ReadPrimitive(PrimitiveType::Scalar(c.rep)));
            },
            Type::Scalar(c) => {
                self.instructions.push(Instruction::ReadPrimitive(PrimitiveType::Scalar(*c)));
            },
            Type::F32 => {
                self.instructions.push(Instruction::ReadPrimitive(PrimitiveType::F32));
            },
            Type::F64 => {
                self.instructions.push(Instruction::ReadPrimitive(PrimitiveType::F64));
            },
            Type::Bool => {
                self.instructions.push(Instruction::ReadPrimitive(PrimitiveType::Bool));
            },
            Type::Ref(field) => {
                self.instructions.push(Instruction::ReadRef(field.clone()));
            }
        }
    }
}