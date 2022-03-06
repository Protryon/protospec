use std::{collections::{HashSet, HashMap}, cell::{Cell, RefCell}};

use crate::Span;

use super::*;

impl Context {

    fn encode_container_refs(&mut self, container: &ContainerType, source: usize) {
        for (name, field) in container.flatten_view() {
            if field.calculated.borrow().is_some() || field.is_pad.get() {
                continue;
            }
            let target = self.alloc_register();
            let type_ = field.type_.borrow();
            let mut ops = vec![];
            if !type_.copyable() {
                ops.push(FieldRef::Ref);
            }
            ops.push(FieldRef::Name(name.clone()));
            self.instructions.push(Instruction::GetField(target, source, ops));
            self.instructions.push(Instruction::SetRef(name, target));
        }
    }

    fn nullcheck_container_refs(&mut self, container: &ContainerType, conditional: bool) {
        if !conditional {
            return;
        }
        for (name, field) in &container.items {
            if field.calculated.borrow().is_some() || field.is_pad.get() || matches!(&*field.type_.borrow(), Type::Container(_)) {
                continue;
            }
            let is_interior_conditional = field.condition.borrow().is_some();
            if !is_interior_conditional {
                let target = self.alloc_register();
                self.instructions.push(Instruction::GetRef(target, name.clone()));
                self.instructions.push(Instruction::NullCheck(target, target, true, "field in optional interior container is None when condition is met".to_string()));
                self.instructions.push(Instruction::SetRef(name.clone(), target));
            }
        }
    }

    fn encode_container_calculated(&mut self, container: &ContainerType) {
        for (name, field) in container.flatten_view() {
            let calculated = field.calculated.borrow();
            if let Some(calculated) = &*calculated {
                if !calculated.blen_calls().is_empty() {
                    continue;
                }
                let calculated_register = self.alloc_register();
                self.instructions.push(Instruction::Eval(calculated_register, calculated.clone()));
                self.instructions.push(Instruction::SetRef(name, calculated_register));
            }
        }
    }

    fn eval_blen_expr(&mut self, blens: &[String], field: &Arc<Field>, calculated: &Expression) -> usize {
        let mut blen_map = HashMap::new();
        for target_name in blens {
            let register = *self.resolved_autos.get(target_name).unwrap();
            let blen_name = format!("__blen_{}", target_name);
            blen_map.insert(target_name.clone(), blen_name.clone());
            self.instructions.push(Instruction::SetRef(blen_name, register));
        }
        let mut calculated = calculated.clone();
        calculated.rewrite_blen_calls(&blen_map);

        let calculated_register = self.alloc_register();
        self.instructions.push(Instruction::Eval(calculated_register, calculated));
        self.instructions.push(Instruction::SetRef(field.name.clone(), calculated_register));
        calculated_register
    }

    fn encode_container_items(&mut self, container: &ContainerType, buf_target: Target, source: usize, conditional: bool) {
        let mut auto_targets = vec![];
        for (name, child) in container.items.iter() {
            let calculated = child.calculated.borrow();
            if let Some(calculated) = &*calculated {
                let blen_calls = calculated.blen_calls();
                if !blen_calls.is_empty() {
                    let mut all_resolved = true;
                    for target in &blen_calls {
                        if !self.resolved_autos.contains_key(target) {
                            all_resolved = false;
                            self.pending_autos.insert(target.clone());
                        }
                    }
                    if all_resolved {
                        self.eval_blen_expr(&blen_calls[..], child, calculated);
                    } else {
                        let new_target = self.alloc_register();
                        self.instructions.push(Instruction::AllocDynBuf(new_target));
                        auto_targets.push((new_target, child));
                        continue;
                    }
                }
            }
            let (real_target, _) = auto_targets
                .last()
                .map(|x| (Target::Buf(x.0), Some(&x.1)))
                .unwrap_or_else(|| (buf_target, None));
            if matches!(&*child.type_.borrow(), Type::Container(_)) || child.is_pad.get() {
                self.encode_field(real_target, source, child, conditional);
            } else {
                let resolved = self.alloc_register();
                self.instructions.push(Instruction::GetRef(resolved, name.clone()));
                self.encode_field(real_target, resolved, child, conditional);
            }

            for (i, (auto_target, auto_field)) in auto_targets.clone().into_iter().enumerate().rev() {
                let calculated = auto_field.calculated.borrow();
                let calculated = calculated.as_ref().unwrap();
                let blen_calls = calculated.blen_calls();
                if blen_calls.iter().all(|x| self.resolved_autos.get(x).is_some()) {
                    let calculated_register = self.eval_blen_expr(&blen_calls[..], auto_field, calculated);

                    auto_targets.remove(i);
                    let target = auto_targets.get(i).map(|(target, _)| Target::Buf(*target)).unwrap_or(buf_target);
                    self.encode_field(target, calculated_register, auto_field, conditional);
                    self.instructions.push(Instruction::EmitBuf(target, auto_target));
                }
            }
        }
        for (_, auto_field) in auto_targets {
            panic!("unused auto field: {}", auto_field.name);
        }
    }

    pub fn encode_container(&mut self, field: &Arc<Field>, type_: &ContainerType, target: Target, source: usize, conditional: bool) {
        let buf_target = if let Some(length) = &type_.length {
            //todo: use limited stream
            let buf = self.alloc_register();
            // we avoid cyclic dependency during encoding by ignoring container length constraint if we're a blen-target
            if self.pending_autos.contains(&field.name) {
                self.instructions.push(Instruction::AllocDynBuf(buf));
            } else {
                let len_register = self.alloc_register();
                self.instructions
                    .push(Instruction::Eval(len_register, length.clone()));
                self.instructions
                    .push(Instruction::AllocBuf(buf, len_register));
            }
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
                    Type::Container(type_) => {
                        let mut unwrapped = vec![];
                        for (subname, subchild) in type_.flatten_view() {
                            if subchild.is_pad.get() || matches!(&*subchild.type_.borrow(), Type::Container(_)) {
                                continue;
                            }
                            let alloced = self.alloc_register();
                            unwrapped.push((
                                subname.clone(),
                                alloced,
                                subchild.type_.borrow().copyable()
                            ));
                        }

                        self.instructions.push(Instruction::UnwrapEnumStruct(
                            field.name.clone(),
                            name.clone(),
                            source,
                            unwrapped.clone(),
                            "mismatch betweeen condition and enum discriminant".to_string(),
                        ));

                        for (name, register, _) in unwrapped {
                            self.instructions.push(Instruction::SetRef(name, register));
                        }
                        self.encode_container_calculated(type_);
                        self.encode_container_items(type_, buf_target, source, false);
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
                        
                        self.encode_field_unconditional(buf_target, unwrapped, child, false, false);
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
            self.instructions.push(Instruction::BreakBlock(drained));

        } else {
            if field.toplevel {
                self.encode_container_refs(type_, source);
                self.encode_container_calculated(type_);    
            } else {
                self.nullcheck_container_refs(type_, conditional);
            }
            self.encode_container_items(type_, buf_target, source, conditional);
        }

        if type_.length.is_some() {
            if self.pending_autos.remove(&field.name) {
                let target = self.alloc_register();
                self.instructions.push(Instruction::GetLen(target, buf_target.unwrap_buf(), Some(ScalarType::U64)));
                self.resolved_autos.insert(field.name.clone(), target);
            }
            self.instructions
                .push(Instruction::EmitBuf(target, buf_target.unwrap_buf()));
        }
    }

}

impl Expression {
    fn rewrite_blen_calls(&mut self, map: &HashMap<String, String>) {
        match self {
            Expression::Binary(expr) => {
                expr.left.rewrite_blen_calls(map);
                expr.right.rewrite_blen_calls(map);
            },
            Expression::Unary(expr) => {
                expr.inner.rewrite_blen_calls(map);
            },
            Expression::Cast(expr) => {
                expr.inner.rewrite_blen_calls(map);
            },
            Expression::ArrayIndex(expr) => {
                expr.array.rewrite_blen_calls(map);
                expr.index.rewrite_blen_calls(map);
            },
            Expression::Ternary(expr) => {
                expr.condition.rewrite_blen_calls(map);
                expr.if_true.rewrite_blen_calls(map);
                expr.if_false.rewrite_blen_calls(map);
            },
            Expression::EnumAccess(_) => (),
            Expression::Int(_) => (),
            Expression::ConstRef(_) => (),
            Expression::InputRef(_) => (),
            Expression::FieldRef(_) => (),
            Expression::Str(_) => (),
            Expression::Bool(_) => (),
            Expression::Call(expr) => {
                for expr in expr.arguments.iter_mut() {
                    expr.rewrite_blen_calls(map);
                }
                if expr.function.name == "blen" {
                    let target = expr.arguments.first().unwrap();
                    let name = match target {
                        Expression::FieldRef(f) => f.name.clone(),
                        _ => panic!("invalid blen target, expected field ref"), // todo: better error message
                    };
                    let new_name = map.get(&name).expect("malformed blen map");
                    *self = Expression::FieldRef(Arc::new(Field {
                        name: new_name.clone(),
                        arguments: RefCell::new(vec![]),
                        span: Span::default(),
                        type_: RefCell::new(Type::Scalar(ScalarType::U64)),
                        calculated: RefCell::new(None),
                        condition: RefCell::new(None),
                        transforms: RefCell::new(vec![]),
                        toplevel: false,
                        is_maybe_cyclical: Cell::new(false),
                        is_pad: Cell::new(false),
                    }));
                }
            },
            Expression::Member(expr) => {
                expr.target.rewrite_blen_calls(map);
            },
        }
    }

    fn blen_calls(&self) -> Vec<String> {
        let mut out = HashSet::new();
        self.extract_magic_blen_calls(&mut out);
        out.into_iter().collect()
    }

    fn extract_magic_blen_calls(&self, output: &mut HashSet<String>) {
        match self {
            Expression::Binary(expr) => {
                expr.left.extract_magic_blen_calls(output);
                expr.right.extract_magic_blen_calls(output);
            },
            Expression::Unary(expr) => {
                expr.inner.extract_magic_blen_calls(output);
            },
            Expression::Cast(expr) => {
                expr.inner.extract_magic_blen_calls(output);
            },
            Expression::ArrayIndex(expr) => {
                expr.array.extract_magic_blen_calls(output);
                expr.index.extract_magic_blen_calls(output);
            },
            Expression::Ternary(expr) => {
                expr.condition.extract_magic_blen_calls(output);
                expr.if_true.extract_magic_blen_calls(output);
                expr.if_false.extract_magic_blen_calls(output);
            },
            Expression::EnumAccess(_) => (),
            Expression::Int(_) => (),
            Expression::ConstRef(_) => (),
            Expression::InputRef(_) => (),
            Expression::FieldRef(_) => (),
            Expression::Str(_) => (),
            Expression::Bool(_) => (),
            Expression::Call(expr) => {
                for expr in &expr.arguments {
                    expr.extract_magic_blen_calls(output);
                }
                if expr.function.name == "blen" {
                    let target = expr.arguments.first().unwrap();
                    output.insert(match target {
                        Expression::FieldRef(f) => f.name.clone(),
                        _ => panic!("invalid blen target, expected field ref"), // todo: better error message
                    });
                }
            },
            Expression::Member(expr) => {
                expr.target.extract_magic_blen_calls(output);
            },
        }
    }
}