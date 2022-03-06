use super::*;
use crate::asg::*;
use std::{collections::HashMap, sync::Arc};

mod instruction;
pub use instruction::*;

mod field;
pub use field::*;

mod array;
pub use array::*;

mod container;
pub use container::*;

#[derive(Debug)]
pub struct Context {
    pub register_count: usize,
    pub field_register_map: HashMap<String, usize>,
    pub instructions: Vec<Instruction>,
    pub name: String,
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
            name: String::new(),
            instructions: vec![],
            field_register_map: HashMap::new(),
            register_count: 0,
        }
    }

    pub fn decode_complex_type(&mut self, source: Target, field: &Arc<Field>) -> Vec<usize> {
        if field.is_pad.get() {
            let array_type = field.type_.borrow();
            let array_type = match &*array_type {
                Type::Array(a) => &**a,
                _ => panic!("invalid type for pad"),
            };
            let len = array_type.length.value.as_ref().cloned().unwrap();
            let length_register = self.alloc_register();
            self.instructions.push(Instruction::Eval(length_register, len, self.field_register_map.clone()));
            self.instructions.push(Instruction::Skip(source, length_register));
            return vec![];
        }
        match &*field.type_.borrow() {
            Type::Container(type_) => self.decode_container(field, &**type_, source),
            type_ => vec![self.decode_type(source, type_)],
        }
    }

    pub fn decode_type(&mut self, source: Target, type_: &Type) -> usize {
        let output = self.alloc_register();
        match type_ {
            Type::Container(_) => {
                unimplemented!("invalid container in non-complex context");
            }
            Type::Array(type_) => self.decode_array(&**type_, source),
            Type::Enum(e) => {
                self.instructions.push(Instruction::DecodeRepr(
                    e.name.clone(),
                    PrimitiveType::Scalar(e.rep.clone()),
                    output,
                    source,
                ));
                output
            }
            Type::Bitfield(e) => {
                self.instructions.push(Instruction::DecodeRepr(
                    e.name.clone(),
                    PrimitiveType::Scalar(e.rep.clone()),
                    output,
                    source,
                ));
                output
            }
            Type::Scalar(s) => {
                self.instructions.push(Instruction::DecodePrimitive(
                    source,
                    output,
                    PrimitiveType::Scalar(*s),
                ));
                output
            }
            Type::F32 => {
                self.instructions.push(Instruction::DecodePrimitive(
                    source,
                    output,
                    PrimitiveType::F32,
                ));
                output
            }
            Type::F64 => {
                self.instructions.push(Instruction::DecodePrimitive(
                    source,
                    output,
                    PrimitiveType::F64,
                ));
                output
            }
            Type::Bool => {
                self.instructions.push(Instruction::DecodePrimitive(
                    source,
                    output,
                    PrimitiveType::Bool,
                ));
                output
            }
            Type::Foreign(f) => {
                self.instructions.push(Instruction::DecodeForeign(
                    source,
                    output,
                    f.clone(),
                    vec![],
                ));
                output
            }
            Type::Ref(r) => {
                let mut args = vec![];
                for arg in r.arguments.iter() {
                    let r = self.alloc_register();
                    self.instructions.push(Instruction::Eval(r, arg.clone(), self.field_register_map.clone()));
                    args.push(r);
                }
                if let Type::Foreign(f) = &*r.target.type_.borrow() {
                    self.instructions.push(Instruction::DecodeForeign(
                        source,
                        output,
                        f.clone(),
                        args,
                    ));
                } else {
                    self.instructions.push(Instruction::DecodeRef(
                        source,
                        output,
                        r.target.name.clone(),
                        args,
                    ));
                }
                output
            }
        }
    }
}
