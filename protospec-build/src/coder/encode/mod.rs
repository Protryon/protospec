use indexmap::{IndexMap, IndexSet};

use super::*;
use crate::asg::*;
use std::sync::Arc;

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

#[derive(Debug)]
pub struct Context {
    pub register_count: usize,
    pub instructions: Vec<Instruction>,
    // map of resolved field name -> register
    pub resolved_autos: IndexMap<String, usize>,
    // set of pending field name
    pub pending_autos: IndexSet<String>,
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
            pending_autos: IndexSet::new(),
        }
    }

    pub fn encode_complex_type(&mut self, field: &Arc<Field>, type_: &Type, target: Target, source: usize, conditional: bool) {
        match type_ {
            Type::Container(type_) => self.encode_container(field, &**type_, target, source, conditional),
            type_ => self.encode_type(type_, target, source),
        }
    }

    pub fn encode_type(&mut self, type_: &Type, target: Target, source: usize) {
        match type_ {
            Type::Container(_) => {
                unimplemented!("invalid container in non-complex context");
            },
            Type::Array(type_) => {
                self.encode_array(type_, target, source);
            }
            Type::Enum(_) => {
                self.instructions.push(Instruction::EncodeEnum(
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
