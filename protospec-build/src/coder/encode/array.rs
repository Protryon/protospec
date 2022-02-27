use super::*;

impl Context {
    pub fn encode_array(&mut self, type_: &ArrayType, target: Target, resolver: &Resolver, source: usize) {
        let terminator = if type_.length.expandable && type_.length.value.is_some() {
            let len = type_.length.value.as_ref().cloned().unwrap();
            let r = self.alloc_register();
            self.instructions.push(Instruction::Eval(r, len));
            Some(r)
        } else {
            None
        };

        let mut len = if terminator.is_none() {
            if let Some(expr) = &type_.length.value {
                self.check_auto(expr, source)
            } else {
                None
            }
        } else {
            None
        };

        if len.is_none() && !type_.length.expandable {
            len = {
                let len = type_.length.value.as_ref().cloned().unwrap();
                let r = self.alloc_register();
                self.instructions.push(Instruction::Eval(r, len));
                Some(r)
            };
        }

        if type_.element.condition.borrow().is_none()
            && type_.element.transforms.borrow().len() == 0
            && terminator.is_none()
        {
            let type_ = type_.element.type_.borrow();
            let type_ = type_.resolved();
            match &*type_ {
                // todo: const-length type optimizations for container/array/foreign
                Type::Container(_) | Type::Array(_) | Type::Foreign(_) | Type::Ref(_) => (),
                Type::Enum(e) => {
                    self.instructions.push(Instruction::EncodePrimitiveArray(
                        target,
                        source,
                        PrimitiveType::Scalar(e.rep),
                        len,
                    ));
                    return;
                },
                Type::Bitfield(e) => {
                    self.instructions.push(Instruction::EncodePrimitiveArray(
                        target,
                        source,
                        PrimitiveType::Scalar(e.rep),
                        len,
                    ));
                    return;
                },
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
        self.instructions.push(Instruction::GetField(
            new_source,
            source,
            vec![FieldRef::ArrayAccess(iter_index)],
        ));
        self.encode_field(target, resolver, new_source, &type_.element);
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
                PrimitiveType::Scalar(ScalarType::U8),
                None,
            ));
        }
    }
}