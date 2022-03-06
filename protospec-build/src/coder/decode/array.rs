use super::*;

impl Context {
    pub fn decode_array(&mut self, type_: &ArrayType, source: Target) -> usize {
        let terminator = if type_.length.expandable && type_.length.value.is_some() {
            let len = type_.length.value.as_ref().cloned().unwrap();
            let r = self.alloc_register();
            self.instructions.push(Instruction::Eval(r, len, self.field_register_map.clone()));
            Some(r)
        } else {
            None
        };

        let len = if type_.length.expandable {
            None
        } else {
            let len = type_.length.value.as_ref().cloned().unwrap();
            let r = self.alloc_register();
            self.instructions.push(Instruction::Eval(r, len, self.field_register_map.clone()));
            Some(r)
        };

        let output = self.alloc_register();
        if terminator.is_none() {
            let type_ = type_.element.resolved();
            match &*type_ {
                // todo: const-length type optimizations for container/array/foreign
                Type::Container(_) | Type::Array(_) | Type::Foreign(_) | Type::Ref(_) => (),
                Type::Enum(x) => {
                    self.instructions.push(Instruction::DecodeReprArray(
                        source,
                        output,
                        x.name.clone(),
                        PrimitiveType::Scalar(x.rep.clone()),
                        len,
                    ));
                    return output;
                }
                Type::Bitfield(x) => {
                    self.instructions.push(Instruction::DecodeReprArray(
                        source,
                        output,
                        x.name.clone(),
                        PrimitiveType::Scalar(x.rep.clone()),
                        len,
                    ));
                    return output;
                }
                Type::Scalar(x) => {
                    self.instructions.push(Instruction::DecodePrimitiveArray(
                        source,
                        output,
                        PrimitiveType::Scalar(*x),
                        len,
                    ));
                    return output;
                }
                Type::F32 => {
                    self.instructions.push(Instruction::DecodePrimitiveArray(
                        source,
                        output,
                        PrimitiveType::F32,
                        len,
                    ));
                    return output;
                }
                Type::F64 => {
                    self.instructions.push(Instruction::DecodePrimitiveArray(
                        source,
                        output,
                        PrimitiveType::F64,
                        len,
                    ));
                    return output;
                }
                Type::Bool => {
                    self.instructions.push(Instruction::DecodePrimitiveArray(
                        source,
                        output,
                        PrimitiveType::Bool,
                        len,
                    ));
                    return output;
                }
            }
        }

        let current_pos = self.instructions.len();
        let item = self.decode_type(source, &*type_.element);
        self.instructions
            .push(Instruction::LoopOutput(output, item));
        let drained = self.instructions.drain(current_pos..).collect();
        self.instructions
            .push(Instruction::Loop(source, len, terminator, output, drained));
        output
    }
}