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

    pub fn decode_type(&mut self, source: Target, field: &Arc<Field>) -> Option<usize> {
        let output = self.alloc_register();
        Some(match &*field.type_.borrow() {
            Type::Container(type_) => {
                return self.decode_container(field, &**type_, source);
            },
            Type::Array(type_) => {
                return self.decode_array(field, &**type_, source);
            }
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
                    self.instructions.push(Instruction::Eval(r, arg.clone()));
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
        })
    }
}
