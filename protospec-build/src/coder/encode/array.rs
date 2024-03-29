use super::*;

impl Context {
    pub fn encode_array(&mut self, type_: &ArrayType, target: Target, source: usize) {
        let terminator = if type_.length.expandable && type_.length.value.is_some() {
            let len = type_.length.value.as_ref().cloned().unwrap();
            let r = self.alloc_register();
            self.instructions.push(Instruction::Eval(r, len));
            Some(r)
        } else {
            None
        };

        // let mut len = if terminator.is_none() {
        //     if let Some(expr) = &type_.length.value {
        //         let target = self.alloc_register();
        //         self.instructions.push(Instruction::GetLen(
        //             target,
        //             source,
        //             Some(ScalarType::U64),
        //         ));
        //         Some(target)
        //     } else {
        //         None
        //     }
        // } else {
        //     None
        // };

        let len = if !type_.length.expandable {
            let len = type_.length.value.as_ref().cloned().unwrap();
            let r = self.alloc_register();
            self.instructions.push(Instruction::Eval(r, len));
            Some(r)
        } else {
            None
        };

        if terminator.is_none() {
            let type_ = type_.element.resolved();
            let primitive_type = match &*type_ {
                // todo: const-length type optimizations for container/array/foreign
                Type::Container(_) | Type::Array(_) | Type::Foreign(_) | Type::Ref(_) => None,
                Type::Enum(e) => {
                    self.instructions.push(Instruction::EncodeReprArray(
                        target,
                        source,
                        PrimitiveType::Scalar(e.rep),
                        len,
                    ));
                    return;
                },
                Type::Bitfield(e) => {
                    self.instructions.push(Instruction::EncodeReprArray(
                        target,
                        source,
                        PrimitiveType::Scalar(e.rep),
                        len,
                    ));
                    return;
                },
                Type::Scalar(s) => Some(PrimitiveType::Scalar(*s)),
                Type::F32 => Some(PrimitiveType::F32),
                Type::F64 => Some(PrimitiveType::F64),
                Type::Bool => Some(PrimitiveType::Bool),
            };
            if let Some(primitive_type) = primitive_type {
                self.instructions.push(Instruction::EncodePrimitiveArray(
                    target,
                    source,
                    primitive_type,
                    len,
                ));
                return;
            }
        }

        let current_pos = self.instructions.len();
        let iter_index = self.alloc_register();
        let new_source = self.alloc_register();

        let mut ops = vec![];
        if !type_.element.copyable() {
            ops.push(FieldRef::Ref);
        }
        ops.push(FieldRef::ArrayAccess(iter_index));
        self.instructions
            .push(Instruction::GetField(new_source, source, ops));
        self.encode_type(&type_.element, target, new_source);
        let drained = self.instructions.drain(current_pos..).collect();
        let len = if let Some(len) = len {
            len
        } else {
            let len = self.alloc_register();
            self.instructions
                .push(Instruction::GetLen(len, source, None));
            len
        };
        self.instructions
            .push(Instruction::Loop(iter_index, len, drained));
        if let Some(terminator) = terminator {
            self.instructions.push(Instruction::EncodePrimitiveArray(
                target,
                terminator,
                PrimitiveType::Scalar(ScalarType::U8.into()),
                None,
            ));
        }
    }
}
