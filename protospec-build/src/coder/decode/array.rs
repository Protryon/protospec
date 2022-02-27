use super::*;

impl Context {
    pub fn decode_array(&mut self, field: &Arc<Field>, type_: &ArrayType, source: Target) -> Option<usize> {
        if field.is_pad.get() {
            let array_type = field.type_.borrow();
            let array_type = match &*array_type {
                Type::Array(a) => &**a,
                _ => panic!("invalid type for pad"),
            };
            let len = array_type.length.value.as_ref().cloned().unwrap();
            let length_register = self.alloc_register();
            self.instructions.push(Instruction::Eval(length_register, len));
            self.instructions.push(Instruction::Skip(source, length_register));
            return None;
        }
        let terminator = if type_.length.expandable && type_.length.value.is_some() {
            let len = type_.length.value.as_ref().cloned().unwrap();
            let r = self.alloc_register();
            self.instructions.push(Instruction::Eval(r, len));
            Some(r)
        } else {
            None
        };

        let len = if type_.length.expandable {
            None
        } else {
            let len = type_.length.value.as_ref().cloned().unwrap();
            let r = self.alloc_register();
            self.instructions.push(Instruction::Eval(r, len));
            Some(r)
        };

        let output = self.alloc_register();
        if type_.element.condition.borrow().is_none()
            && type_.element.transforms.borrow().len() == 0
            && terminator.is_none()
        {
            let type_ = type_.element.type_.borrow();
            let type_ = type_.resolved();
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
                    return Some(output);
                }
                Type::Bitfield(x) => {
                    self.instructions.push(Instruction::DecodeReprArray(
                        source,
                        output,
                        x.name.clone(),
                        PrimitiveType::Scalar(x.rep.clone()),
                        len,
                    ));
                    return Some(output);
                }
                Type::Scalar(x) => {
                    self.instructions.push(Instruction::DecodePrimitiveArray(
                        source,
                        output,
                        PrimitiveType::Scalar(*x),
                        len,
                    ));
                    return Some(output);
                }
                Type::F32 => {
                    self.instructions.push(Instruction::DecodePrimitiveArray(
                        source,
                        output,
                        PrimitiveType::F32,
                        len,
                    ));
                    return Some(output);
                }
                Type::F64 => {
                    self.instructions.push(Instruction::DecodePrimitiveArray(
                        source,
                        output,
                        PrimitiveType::F64,
                        len,
                    ));
                    return Some(output);
                }
                Type::Bool => {
                    self.instructions.push(Instruction::DecodePrimitiveArray(
                        source,
                        output,
                        PrimitiveType::Bool,
                        len,
                    ));
                    return Some(output);
                }
            }
        }

        let current_pos = self.instructions.len();
        let item = self.decode_field(source, &type_.element);
        if item.is_none() {
            unimplemented!("cannot have inline container inside array");
        }
        self.instructions
            .push(Instruction::LoopOutput(output, item.unwrap()));
        let drained = self.instructions.drain(current_pos..).collect();
        self.instructions
            .push(Instruction::Loop(source, len, terminator, output, drained));
        Some(output)
    }
}