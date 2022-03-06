use super::*;

impl Context {
    pub fn decode_container(&mut self, field: &Arc<Field>, type_: &ContainerType, source: Target) -> Vec<usize> {
        let buf_target = if let Some(length) = &type_.length {
            //todo: use limited stream
            let len_register = self.alloc_register();
            self.instructions
                .push(Instruction::Eval(len_register, length.clone(), self.field_register_map.clone()));
            let buf = self.alloc_register();
            self.instructions
                .push(Instruction::Constrict(source, buf, len_register));
            Target::Stream(buf)
        } else {
            source
        };
        if type_.is_enum.get() {
            self.decode_enum_container(field, type_, buf_target)
        } else {
            self.decode_struct_container(field, type_, buf_target)
        }
    }

    fn decode_struct_container(&mut self, field: &Arc<Field>, type_: &ContainerType, buf_target: Target) -> Vec<usize> {
        let mut decoded_fields = vec![];
        for (name, child) in type_.items.iter() {
            let decoded = self.decode_field(buf_target, child);
            decoded_fields.extend_from_slice(&decoded[..]);
            if !matches!(&*child.type_.borrow(), Type::Container(_)) {
                for decoded in decoded {
                    self.field_register_map.insert(name.clone(), decoded);
                }
            }
        }
        if !field.toplevel {
            return decoded_fields;
        }
        let emitted = self.alloc_register();
        let mut items = vec![];
        for (name, child) in type_.flatten_view() {
            if child.is_pad.get() || matches!(&*child.type_.borrow(), Type::Container(_)) {
                continue;
            }
            items.push((
                name.clone(),
                *self
                    .field_register_map
                    .get(&name)
                    .expect("missing field in field_register_map"),
            ));
        }
        self.instructions.push(Instruction::Construct(
            emitted,
            Constructable::Struct {
                name: field.name.clone(),
                items,
            },
        ));
        vec![emitted]
    }

    fn decode_enum_container(&mut self, field: &Arc<Field>, type_: &ContainerType, buf_target: Target) -> Vec<usize> {
        assert!(type_.is_enum.get());
        for (name, child) in type_.items.iter() {
            let condition = self.decode_field_condition(child);
            let start = self.instructions.len();            
            let decoded = self.decode_field_unconditional(buf_target, child);
            let target = self.alloc_register();
            
            let subtype = child.type_.borrow();
            match &*subtype {
                Type::Container(c) => {
                    let mut values = vec![];
                    for (subname, subchild) in c.flatten_view() {
                        if subchild.is_pad.get() || matches!(&*subchild.type_.borrow(), Type::Container(_)) {
                            continue;
                        }

                        values.push((
                            subname.clone(),
                            *self
                                .field_register_map
                                .get(&subname)
                                .expect("missing field in field_register_map"),
                        ));
                    }

                    self.instructions.push(Instruction::Construct(target, Constructable::TaggedEnumStruct {
                        name: field.name.clone(),
                        discriminant: name.clone(),
                        values,
                    }));
                },
                _ => {
                    let decoded = decoded.first().expect("enum discriminant was proper interior container, which is illegal");
                    self.instructions.push(Instruction::Construct(target, Constructable::TaggedEnum {
                        name: field.name.clone(),
                        discriminant: name.clone(),
                        values: vec![*decoded],
                    }));
                },
            }

            if let Some(condition) = condition {
                self.instructions.push(Instruction::Return(target));
                let drained = self.instructions.drain(start..).collect();
                self.instructions.push(Instruction::ConditionalPredicate(
                    condition,
                    drained,
                ));
            } else {
                return vec![target];
            }
        }
        self.instructions.push(Instruction::Error(format!("no enum conditions matched for {}", field.name)));
        vec![]
    }
}