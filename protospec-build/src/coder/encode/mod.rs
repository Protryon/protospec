use indexmap::IndexMap;

use super::*;
use crate::asg::*;
use std::{sync::Arc, collections::HashMap};

mod instruction;
pub use instruction::*;

mod field;
pub use field::*;

mod container;
pub use container::*;

mod array;
pub use array::*;

mod var_ref;
pub use var_ref::*;

mod auto;
pub use auto::*;

type Resolver = Box<dyn Fn(&mut Context, &str) -> usize>;

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

    pub fn encode_type(&mut self, field: &Arc<Field>, type_: &Type, target: Target, resolver: &Resolver, source: usize) {
        match type_ {
            Type::Container(type_) => {
                self.encode_container(field, &**type_, target, resolver, source);
            },
            Type::Array(type_) => {
                self.encode_array(type_, target, resolver, source);
            }
            Type::Enum(e) => {
                self.instructions.push(Instruction::EncodeEnum(
                    PrimitiveType::Scalar(e.rep.clone()),
                    target,
                    source,
                ));
            }
            Type::Bitfield(_) => {
                self.instructions.push(Instruction::EncodeBitfield(
                    target,
                    source,
                ));
            }
            Type::Scalar(s) => {
                self.instructions.push(Instruction::EncodePrimitive(
                    target,
                    source,
                    PrimitiveType::Scalar(*s),
                ));
            }
            Type::F32 => {
                self.instructions.push(Instruction::EncodePrimitive(
                    target,
                    source,
                    PrimitiveType::F32,
                ));
            }
            Type::F64 => {
                self.instructions.push(Instruction::EncodePrimitive(
                    target,
                    source,
                    PrimitiveType::F64,
                ));
            }
            Type::Bool => {
                self.instructions.push(Instruction::EncodePrimitive(
                    target,
                    source,
                    PrimitiveType::Bool,
                ));
            }
            Type::Foreign(f) => {
                self.instructions.push(Instruction::EncodeForeign(
                    target,
                    source,
                    f.clone(),
                    vec![],
                ));
            }
            Type::Ref(type_) => {
                self.encode_var_ref(type_, target, source);
            }
        }
    }
}
