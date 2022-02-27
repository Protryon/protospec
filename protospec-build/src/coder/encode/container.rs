use super::*;

impl Context {

    pub fn encode_container_items(&mut self, container: &ContainerType, buf_target: Target, resolver: &Resolver, source: usize) {
        let mut auto_targets = vec![];
        for (name, child) in container.items.iter() {
            if child.is_auto.get() {
                let new_target = self.alloc_register();
                self.instructions.push(Instruction::AllocDynBuf(new_target));
                auto_targets.push((new_target, child));
                continue;
            }
            let (real_target, _) = auto_targets
                .last()
                .map(|x| (Target::Buf(x.0), Some(&x.1)))
                .unwrap_or_else(|| (buf_target, None));
            if matches!(&*child.type_.borrow(), Type::Container(_)) || child.is_pad.get() {
                self.encode_field(real_target, resolver, source, child);
            } else {
                let resolved = resolver(self, &**name);
                self.encode_field(real_target, resolver, resolved, child);
            }

            for (i, (auto_target, auto_field)) in auto_targets.clone().into_iter().enumerate().rev() {
                if let Some(resolved) = self.resolved_autos.get(&auto_field.name).copied() {
                    auto_targets.remove(i);
                    let target = auto_targets.get(i).map(|(target, _)| Target::Buf(*target)).unwrap_or(buf_target);
                    self.encode_field(target, resolver, resolved, auto_field);
                    self.instructions
                        .push(Instruction::EmitBuf(target, auto_target));
                }
            }
        }
        for (_, auto_field) in auto_targets {
            panic!("unused auto field: {}", auto_field.name);
        }
    }

    pub fn encode_container(&mut self, field: &Arc<Field>, type_: &ContainerType, target: Target, resolver: &Resolver, source: usize) {
        let buf_target = if let Some(length) = &type_.length {
            //todo: use limited stream
            let len_register = self.alloc_register();
            let buf = self.alloc_register();
            self.instructions
                .push(Instruction::Eval(len_register, length.clone()));
            self.instructions
                .push(Instruction::AllocBuf(buf, len_register));
            Target::Buf(buf)
        } else {
            target
        };
        if type_.is_enum.get() {
            let break_start = self.instructions.len();
            for (name, child) in type_.items.iter() {
                let condition = self.encode_field_condition(child);
                let start = self.instructions.len();
                let unwrapped = self.alloc_register();

                let subtype = child.type_.borrow();
                match &*subtype {
                    Type::Container(c) => {

                        let mut unwrapped = vec![];
                        for (subname, subchild) in c.flatten_view() {
                            if subchild.is_pad.get() || matches!(&*subchild.type_.borrow(), Type::Container(_)) {
                                continue;
                            }
                            let alloced = self.alloc_register();
                            unwrapped.push((
                                subname.clone(),
                                alloced,
                            ));
                        }

                        self.instructions.push(Instruction::UnwrapEnumStruct(
                            field.name.clone(),
                            name.clone(),
                            source,
                            unwrapped.clone(),
                            "mismatch betweeen condition and enum discriminant".to_string(),
                        ));

                        let map = unwrapped.into_iter().collect::<HashMap<_, _>>();

                        let resolver: Resolver = Box::new(move |_context, name| *map.get(name).expect("illegal field ref"));
                        self.encode_container_items(c, buf_target, &resolver, source);
                        self.instructions.push(Instruction::Break);
                    },
                    _ => {
                        self.instructions.push(Instruction::UnwrapEnum(
                            field.name.clone(),
                            name.clone(),
                            source,
                            unwrapped,
                            "mismatch betweeen condition and enum discriminant".to_string(),
                        ));
                        
                        let resolver: Resolver = Box::new(|_, _| panic!("fields refs illegal in raw enum value"));
                        self.encode_field_unconditional(buf_target, &resolver, unwrapped, child, false);
                        self.instructions.push(Instruction::Break);
                    },
                }

                if let Some(condition) = condition {
                    let drained = self.instructions.drain(start..).collect();
                    self.instructions
                        .push(Instruction::Conditional(condition, drained, vec![]));
                }
            }
            let drained = self.instructions.drain(break_start..).collect();
            self.instructions
                .push(Instruction::BreakBlock(drained));

        } else {
            self.encode_container_items(type_, buf_target, resolver, source);
        }

        if let Some(length) = &type_.length {
            self.check_auto(length, buf_target.unwrap_buf());
            self.instructions
                .push(Instruction::EmitBuf(target, buf_target.unwrap_buf()));
        }
    }

}