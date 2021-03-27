use indexmap::IndexMap;

use crate::{asg::*};
use std::sync::Arc;
use super::*;

#[derive(Debug)]
pub enum Instruction {
    Eval(usize, Expression),
    GetField(usize, usize, Vec<FieldRef>), // dest, source, op
    AllocBuf(usize, usize), // buf handle, len handle
    AllocDynBuf(usize), // buf handle
    WrapStream(Target, usize, Arc<Transform>, Vec<usize>), // stream, new stream, transformer, arguments
    ConditionalWrapStream(usize, Vec<Instruction>, Target, usize, usize, Arc<Transform>, Vec<usize>), // condition, prelude, stream, new stream, owned_new_stream, transformer, arguments
    ProxyStream(Target, usize), // stream, new stream
    EndStream(usize),
    
    EmitBuf(Target, usize),

    EncodeForeign(Target, usize, Arc<NamedForeignType>, Vec<usize>),
    EncodeRef(Target, usize, Vec<usize>),
    EncodeEnum(PrimitiveType, Target, usize),
    EncodePrimitive(Target, usize, PrimitiveType),
    EncodePrimitiveArray(Target, usize, PrimitiveType, Option<usize>),

    // register representing iterator from -> term, term, inner
    Loop(usize, usize, Vec<Instruction>),
    // len target <- buffer
    GetLen(usize, usize),
    Drop(usize),
    // original, checked, message
    NullCheck(usize, usize, String),
    Conditional(usize, Vec<Instruction>, Vec<Instruction>), // condition, if_true, if_false
}

#[derive(Debug)]
pub struct Context {
    pub register_count: usize,
    pub instructions: Vec<Instruction>,
    pub resolved_autos: IndexMap<String, usize>,
}

impl Context {
    fn alloc_register(&mut self) -> usize {
        let x = self.register_count;
        self.register_count += 1;
        x
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            instructions: vec![],
            register_count: 0,
            resolved_autos: IndexMap::new(),
        }
    }

    pub fn encode_field_top(&mut self, field: &Arc<Field>) {
        let top = self.alloc_register(); // implicitly set to self/equivalent
        match &field.type_ {
            Type::Foreign(_) => return,
            Type::Container(_) => (),
            Type::Enum(_) => (),
            _ => {
                self.instructions.push(Instruction::GetField(0, 0, vec![FieldRef::TupleAccess(0)]))
            },
        }
        self.encode_field(Target::Direct, top, top, field);
    }

    pub fn encode_field(&mut self, mut target: Target, root: usize, source: usize, field: &Arc<Field>) {
        let field_condition = if let Some(condition) = field.condition.borrow().as_ref() {
            let value = self.alloc_register();
            self.instructions.push(Instruction::Eval(value, condition.clone()));
            Some(value)
        } else {
            None
        };
        let start = self.instructions.len();
        let mut new_streams = vec![];

        for transform in field.transforms.borrow().iter() {
            let condition = if let Some(condition) = &transform.condition {
                let value = self.alloc_register();
                self.instructions.push(Instruction::Eval(value, condition.clone()));
                Some(value)
            } else {
                None
            };

            let argument_start = self.instructions.len();
            let mut args = vec![];
            for arg in transform.arguments.iter() {
                let r = self.alloc_register();
                self.instructions.push(Instruction::Eval(r, arg.clone()));
                args.push(r);
            }
            let new_stream = self.alloc_register();
            let new_owned_stream = condition.map(|_| self.alloc_register());
            new_streams.push((new_stream, new_owned_stream));

            if let Some(condition) = condition {
                let drained = self.instructions.drain(argument_start..).collect();
                self.instructions.push(Instruction::ConditionalWrapStream(condition, drained, target, new_stream, new_owned_stream.unwrap(), transform.transform.clone(), args));
            } else {
                self.instructions.push(Instruction::WrapStream(target, new_stream, transform.transform.clone(), args));
            }
            target = Target::Stream(new_stream);
        }

        // let (buf_target, buf) = if field.transforms.borrow().len() > 0 {
        //     let buf = self.alloc_register();
        //     self.instructions.push(Instruction::AllocDynBuf(buf));
        //     (Target::Buf(buf), Some(buf))
        // } else {
        //     (target, None)
        // };
        let source = if field_condition.is_some() {
            let real_source = self.alloc_register();
            self.instructions.push(Instruction::NullCheck(source, real_source, "failed null check for conditional field".to_string()));
            real_source
        } else {
            source
        };
        
        match &field.type_ {
            Type::Container(c) => {
                let buf_target = if let Some(length) = &c.length {
                    //todo: use limited stream
                    let len_register = self.alloc_register();
                    let buf = self.alloc_register();
                    self.instructions.push(Instruction::Eval(len_register, length.clone()));
                    self.instructions.push(Instruction::AllocBuf(buf, len_register));
                    Target::Buf(buf)
                } else {
                    target
                };
                let mut auto_target = vec![];
                for (name, child) in c.items.iter() {
                    if child.is_auto {
                        let new_target = self.alloc_register();
                        self.instructions.push(Instruction::AllocDynBuf(new_target));
                        auto_target.push((new_target, child));
                        continue;
                    }
                    let (real_target, auto_field) = auto_target.last().map(|x| (Target::Buf(x.0), Some(&x.1))).unwrap_or_else(|| (buf_target, None));
                    if matches!(child.type_, Type::Container(_)) {
                        self.encode_field(real_target, root, source, child);
                    } else {
                        let value = self.alloc_register();
                        self.instructions.push(Instruction::GetField(value, root, vec![FieldRef::Name(name.clone())]));
                        self.encode_field(real_target, root, value, child);        
                    }
                    if let Some(auto_field) = auto_field {
                        if let Some(resolved) = self.resolved_autos.get(&auto_field.name).copied() {
                            let auto_field = *auto_field;
                            let (auto_target, _) = auto_target.pop().unwrap();
                            self.encode_field(buf_target, root, resolved, auto_field);
                            self.instructions.push(Instruction::EmitBuf(buf_target, auto_target));
                        }
                    }
                }
                if let Some(length) = &c.length {
                    match length {
                        Expression::FieldRef(f) if f.is_auto => {
                            let target = self.alloc_register();
                            self.instructions.push(Instruction::GetLen(target, buf_target.unwrap_buf()));
                            self.resolved_autos.insert(f.name.clone(), target);
                        },
                        _ => (),
                    }
                    self.instructions.push(Instruction::EmitBuf(target, buf_target.unwrap_buf()));
                }
            },
            t => self.encode_type(target, source, t),
        }

        for (stream, owned_stream) in new_streams.iter().rev() {
            self.instructions.push(Instruction::EndStream(*stream));
            if let Some(owned_stream) = owned_stream {
                self.instructions.push(Instruction::Drop(*owned_stream));
            }
        }

        // for transform in field.transforms.borrow().iter() {
        //     let condition = if let Some(condition) = &transform.condition {
        //         let value = self.alloc_register();
        //         self.instructions.push(Instruction::Eval(value, condition.clone()));
        //         Some(value)
        //     } else {
        //         None
        //     };

        //     let transform_start = self.instructions.len();
        //     let mut args = vec![];
        //     for arg in transform.arguments.iter() {
        //         let r = self.alloc_register();
        //         self.instructions.push(Instruction::Eval(r, arg.clone()));
        //         args.push(r);
        //     }
        //     self.instructions.push(Instruction::MutateBuf(buf.unwrap(), transform.transform.clone(), args));
        //     if let Some(condition) = condition {
        //         let drained = self.instructions.drain(transform_start..).collect();
        //         self.instructions.push(Instruction::Conditional(condition, drained));
        //     }
        // }

        if let Some(field_condition) = field_condition {
            let drained = self.instructions.drain(start..).collect();
            self.instructions.push(Instruction::Conditional(field_condition, drained, vec![]));
        }
        // if let Some(buf) = buf {
        //     self.instructions.push(Instruction::EmitBuf(target, buf));
        // }
    }

    pub fn encode_type(&mut self, target: Target, source: usize, type_: &Type) {
        match type_ {
            Type::Container(_) => unimplemented!(),
            Type::Array(c) => {
                if c.length.expandable && c.length.value.is_some() {
                    todo!()
                }
            
                let len = if c.length.expandable {
                    None
                } else {
                    let len = c.length.value.as_ref().cloned().unwrap();
                    let r = self.alloc_register();
                    self.instructions.push(Instruction::Eval(r, len));
                    Some(r)
                };
                

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
                            self.instructions.push(Instruction::EncodePrimitiveArray(
                                target,
                                source,
                                PrimitiveType::Scalar(x.rep),
                                len,
                            ));
                            return;
                        }
                        Type::Scalar(x) => {
                            self.instructions.push(Instruction::EncodePrimitiveArray(
                                target,
                                source,
                                PrimitiveType::Scalar(*x),
                                len,
                            ));
                            return;
                        }
                        Type::F32 => {
                            self.instructions.push(Instruction::EncodePrimitiveArray(
                                target,
                                source,
                                PrimitiveType::F32,
                                len,
                            ));
                            return;
                        }
                        Type::F64 => {
                            self.instructions.push(Instruction::EncodePrimitiveArray(
                                target,
                                source,
                                PrimitiveType::F64,
                                len,
                            ));
                            return;
                        }
                        Type::Bool => {
                            self.instructions.push(Instruction::EncodePrimitiveArray(
                                target,
                                source,
                                PrimitiveType::Bool,
                                len,
                            ));
                            return;
                        }
                    }
                }

                let current_pos = self.instructions.len();
                let iter_index = self.alloc_register();
                let new_source = self.alloc_register();
                self.instructions.push(Instruction::GetField(new_source, source, vec![FieldRef::ArrayAccess(iter_index)]));
                self.encode_field(target, 0, new_source, &c.element);
                let drained = self.instructions.drain(current_pos..).collect();
                let len = if let Some(len) = len {
                    len
                } else {
                    let len = self.alloc_register();
                    self.instructions.push(Instruction::GetLen(len, source));
                    len
                };
                self.instructions.push(Instruction::Loop(iter_index, len, drained));
            },
            Type::Enum(e) => {
                self.instructions.push(Instruction::EncodeEnum(PrimitiveType::Scalar(e.rep.clone()), target, source));
            },
            Type::Scalar(s) => {
                self.instructions.push(Instruction::EncodePrimitive(target, source, PrimitiveType::Scalar(*s)));
            },
            Type::F32 => {
                self.instructions.push(Instruction::EncodePrimitive(target, source, PrimitiveType::F32));
            },
            Type::F64 => {
                self.instructions.push(Instruction::EncodePrimitive(target, source, PrimitiveType::F64));
            },
            Type::Bool => {
                self.instructions.push(Instruction::EncodePrimitive(target, source, PrimitiveType::Bool));
            },
            Type::Foreign(f) => {
                self.instructions.push(Instruction::EncodeForeign(target, source, f.clone(), vec![]));
            },
            Type::Ref(r) => {
                let mut args = vec![];
                for arg in r.arguments.iter() {
                    let r = self.alloc_register();
                    self.instructions.push(Instruction::Eval(r, arg.clone()));
                    args.push(r);
                }
                if let Type::Foreign(f) = &r.target.type_ {
                    self.instructions.push(Instruction::EncodeForeign(target, source, f.clone(), args));
                } else {
                    self.instructions.push(Instruction::EncodeRef(target, source, args));
                }
            },
        }
    }
}
