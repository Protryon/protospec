use super::*;

impl Context {

    pub fn decode_field_top(&mut self, field: &Arc<Field>) {
        assert!(field.toplevel);
        self.name = field.name.clone();
        let mut value = self.decode_field(Target::Direct, field);
        if value.len() > 1 {
            panic!("leaked poly-value to top level field");
        }
        match &*field.type_.borrow() {
            Type::Foreign(_) => (),
            Type::Container(_) => (),
            Type::Enum(_) => (),
            Type::Bitfield(_) => (),
            _ => {
                if let Some(old_value) = value.first() {
                    let extra_value = self.alloc_register();
                    self.instructions.push(Instruction::Construct(
                        extra_value,
                        Constructable::TaggedTuple {
                            name: field.name.clone(),
                            items: vec![*old_value],
                        },
                    ));
                    value = vec![extra_value];
                }
            }
        }
        if let Some(value) = value.first() {
            self.instructions.push(Instruction::Return(
                *value,
            ));    
        }
    }

    pub fn decode_field_condition(&mut self, field: &Arc<Field>) -> Option<usize> {
        if let Some(condition) = field.condition.borrow().as_ref() {
            let value = self.alloc_register();
            self.instructions
                .push(Instruction::Eval(value, condition.clone(), self.field_register_map.clone()));
            Some(value)
        } else {
            None
        }
    }

    /// return of `None` means interior container
    pub fn decode_field(&mut self, source: Target, field: &Arc<Field>) -> Vec<usize> {
        let field_condition = self.decode_field_condition(field);
        let start = self.instructions.len();
        
        let emitted = self.decode_field_unconditional(source, field);

        if field.is_pad.get() {
            return emitted;
        }
        if let Some(field_condition) = field_condition {
            let drained = self.instructions.drain(start..).collect();
            if emitted.is_empty() {
                panic!("no emitted values from conditional");
            }
            self.instructions.push(Instruction::Conditional(
                emitted.clone(),
                emitted.clone(),
                field_condition,
                drained,
            ));
        }
        emitted
    }

    pub fn decode_field_unconditional(&mut self, mut source: Target, field: &Arc<Field>) -> Vec<usize> {
        let mut new_streams = vec![];

        for transform in field.transforms.borrow().iter().rev() {
            let condition = if let Some(condition) = &transform.condition {
                let value = self.alloc_register();
                self.instructions
                    .push(Instruction::Eval(value, condition.clone(), self.field_register_map.clone()));
                Some(value)
            } else {
                None
            };

            let argument_start = self.instructions.len();
            let mut args = vec![];
            for arg in transform.arguments.iter() {
                let r = self.alloc_register();
                self.instructions.push(Instruction::Eval(r, arg.clone(), self.field_register_map.clone()));
                args.push(r);
            }
            let new_stream = self.alloc_register();
            new_streams.push(new_stream);

            if let Some(condition) = condition {
                let drained = self.instructions.drain(argument_start..).collect();
                self.instructions.push(Instruction::ConditionalWrapStream(
                    condition,
                    drained,
                    source,
                    new_stream,
                    transform.transform.clone(),
                    args,
                ));
            } else {
                self.instructions.push(Instruction::WrapStream(
                    source,
                    new_stream,
                    transform.transform.clone(),
                    args,
                ));
            }
            source = Target::Stream(new_stream);
        }

        //todo: assert condition matching actual presence
        let emitted = self.decode_complex_type(source, field);

        emitted
    }
}