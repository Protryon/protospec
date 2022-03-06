use super::*;

impl Context {

    pub fn encode_field_top(&mut self, field: &Arc<Field>) {
        let top = self.alloc_register(); // implicitly set to self/equivalent
        match &*field.type_.borrow() {
            Type::Foreign(_) => return,
            Type::Container(_) => (),
            Type::Enum(_) => (),
            Type::Bitfield(_) => (),
            type_ => {
                let mut ops = vec![];
                if !type_.copyable() {
                    ops.push(FieldRef::Ref);
                }
                ops.push(FieldRef::TupleAccess(0));
    
                self.instructions
                    .push(Instruction::GetField(0, 0, ops))
            }
        }
        self.encode_field(Target::Direct, top, field, false);
    }

    pub fn encode_field_condition(&mut self, field: &Arc<Field>) -> Option<usize> {
        if let Some(condition) = field.condition.borrow().as_ref() {
            let value = self.alloc_register();
            self.instructions
                .push(Instruction::Eval(value, condition.clone()));
            Some(value)
        } else {
            None
        }
    }

    pub fn encode_field(
        &mut self,
        target: Target,
        source: usize,
        field: &Arc<Field>,
        conditional: bool,
    ) {
        let field_condition = self.encode_field_condition(field);
        let start = self.instructions.len();
        
        self.encode_field_unconditional(target, source, field, field_condition.is_some(), field_condition.is_some() || conditional);

        if let Some(field_condition) = field_condition {
            let drained = self.instructions.drain(start..).collect();
            self.instructions
                .push(Instruction::Conditional(field_condition, drained, vec![]));
        }
    }


    pub fn encode_field_unconditional(
        &mut self,
        mut target: Target,
        source: usize,
        field: &Arc<Field>,
        self_conditional: bool,
        total_conditional: bool,
    ) {
        let mut new_streams = vec![];

        for transform in field.transforms.borrow().iter() {
            let condition = if let Some(condition) = &transform.condition {
                let value = self.alloc_register();
                self.instructions
                    .push(Instruction::Eval(value, condition.clone()));
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
                self.instructions.push(Instruction::ConditionalWrapStream(
                    condition,
                    drained,
                    target,
                    new_stream,
                    new_owned_stream.unwrap(),
                    transform.transform.clone(),
                    args,
                ));
            } else {
                self.instructions.push(Instruction::WrapStream(
                    target,
                    new_stream,
                    transform.transform.clone(),
                    args,
                ));
            }
            target = Target::Stream(new_stream);
        }

        let is_psuedocontainer = !field.toplevel && matches!(&*field.type_.borrow(), Type::Container(_));

        let source = if self_conditional && !is_psuedocontainer && !field.calculated.borrow().is_some() && !field.is_pad.get() {
            let real_source = self.alloc_register();
            self.instructions.push(Instruction::NullCheck(
                source,
                real_source,
                field.type_.borrow().copyable(),
                "failed null check for conditional field".to_string(),
            ));
            real_source
        } else {
            source
        };

        match &*field.type_.borrow() {
            _ if field.is_pad.get() => {
                let array_type = field.type_.borrow();
                let array_type = match &*array_type {
                    Type::Array(a) => &**a,
                    _ => panic!("invalid type for pad"),
                };
                let len = array_type.length.value.as_ref().cloned().unwrap();
                let length_register = self.alloc_register();
                self.instructions.push(Instruction::Eval(length_register, len));
                self.instructions.push(Instruction::Pad(target, length_register));
            },
            type_ => self.encode_complex_type(field, type_, target, source, total_conditional),
        }

        for (stream, owned_stream) in new_streams.iter().rev() {
            self.instructions.push(Instruction::EndStream(*stream));
            if let Some(owned_stream) = owned_stream {
                self.instructions.push(Instruction::Drop(*owned_stream));
            }
        }
    }
}