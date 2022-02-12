use indexmap::IndexMap;

use super::*;
use crate::asg::*;
use std::{sync::Arc, collections::HashMap};

#[derive(Debug)]
pub enum Instruction {
    Eval(usize, Expression),
    GetField(usize, usize, Vec<FieldRef>), // dest, source, op
    AllocBuf(usize, usize),                // buf handle, len handle
    AllocDynBuf(usize),                    // buf handle
    WrapStream(Target, usize, Arc<Transform>, Vec<usize>), // stream, new stream, transformer, arguments
    ConditionalWrapStream(
        usize,
        Vec<Instruction>,
        Target,
        usize,
        usize,
        Arc<Transform>,
        Vec<usize>,
    ), // condition, prelude, stream, new stream, owned_new_stream, transformer, arguments
    ProxyStream(Target, usize),                            // stream, new stream
    EndStream(usize),

    EmitBuf(Target, usize),

    EncodeForeign(Target, usize, Arc<ForeignType>, Vec<usize>),
    EncodeRef(Target, usize, Vec<usize>),
    EncodeEnum(PrimitiveType, Target, usize),
    EncodePrimitive(Target, usize, PrimitiveType),
    EncodePrimitiveArray(Target, usize, PrimitiveType, Option<usize>),

    // register representing iterator from -> term, term, inner
    Loop(usize, usize, Vec<Instruction>),
    // len target <- buffer, cast_type
    GetLen(usize, usize, Option<ScalarType>),
    Drop(usize),
    // original, checked, message
    NullCheck(usize, usize, String),
    Conditional(usize, Vec<Instruction>, Vec<Instruction>), // condition, if_true, if_false
    /// enum name, discriminant, original, checked, message
    UnwrapEnum(String, String, usize, usize, String),
    /// enum name, discriminant, original, checked: (enumstruct field name, checked), message
    UnwrapEnumStruct(String, String, usize, Vec<(String, usize)>, String),
    BreakBlock(Vec<Instruction>),
    Break,
}

type Resolver = Box<dyn Fn(&mut Context, &str) -> usize>;

#[derive(Debug)]
pub struct Context {
    pub register_count: usize,
    pub instructions: Vec<Instruction>,
    pub resolved_autos: IndexMap<String, usize>,
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
        }
    }

    pub fn encode_field_top(&mut self, field: &Arc<Field>) {
        let top = self.alloc_register(); // implicitly set to self/equivalent
        match &*field.type_.borrow() {
            Type::Foreign(_) => return,
            Type::Container(_) => (),
            Type::Enum(_) => (),
            _ => {
                self.instructions
                    .push(Instruction::GetField(0, 0, vec![FieldRef::TupleAccess(0)]))
            }
        }
        let resolver: Resolver = Box::new(move |context: &mut Context, name: &str| {
            let value = context.alloc_register();
            context.instructions.push(Instruction::GetField(
                value,
                top,
                vec![FieldRef::Name(name.to_string())],
            ));
            value
        });
        self.encode_field(Target::Direct, &resolver, top, field);
    }

    fn encode_field_condition(&mut self, field: &Arc<Field>) -> Option<usize> {
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
        resolver: &Resolver,
        source: usize,
        field: &Arc<Field>,
    ) {
        let field_condition = self.encode_field_condition(field);
        let start = self.instructions.len();
        
        self.encode_field_unconditional(target, resolver, source, field, field_condition.is_some());

        if let Some(field_condition) = field_condition {
            let drained = self.instructions.drain(start..).collect();
            self.instructions
                .push(Instruction::Conditional(field_condition, drained, vec![]));
        }
    }

    fn encode_container_items(&mut self, container: &ContainerType, buf_target: Target, resolver: &Resolver, source: usize) {
        let mut auto_target = vec![];
        for (name, child) in container.items.iter() {
            if child.is_auto.get() {
                let new_target = self.alloc_register();
                self.instructions.push(Instruction::AllocDynBuf(new_target));
                auto_target.push((new_target, child));
                continue;
            }
            let (real_target, auto_field) = auto_target
                .last()
                .map(|x| (Target::Buf(x.0), Some(&x.1)))
                .unwrap_or_else(|| (buf_target, None));
            if matches!(&*child.type_.borrow(), Type::Container(_)) {
                //TODO: this may not work
                self.encode_field(real_target, resolver, source, child);
            } else {
                let resolved = resolver(self, &**name);
                self.encode_field(real_target, resolver, resolved, child);
            }
            if let Some(auto_field) = auto_field {
                if let Some(resolved) = self.resolved_autos.get(&auto_field.name).copied() {
                    let auto_field = *auto_field;
                    let (auto_target, target_auto_field) = auto_target.pop().unwrap();
                    if auto_field.name != target_auto_field.name {
                        panic!("+auto tags must be declared and used in a hierarchical manner");
                    }
                    self.encode_field(buf_target, resolver, resolved, auto_field);
                    self.instructions
                        .push(Instruction::EmitBuf(buf_target, auto_target));
                } else {
                    panic!("unresolved +auto field: {}", auto_field.name);
                }
            }
        }
        for (_, auto_field) in auto_target {
            panic!("unused auto field: {}", auto_field.name);
        }
    }

    fn encode_field_unconditional(
        &mut self,
        mut target: Target,
        resolver: &Resolver,
        source: usize,
        field: &Arc<Field>,
        was_conditional: bool,
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

        let source = if was_conditional {
            let real_source = self.alloc_register();
            self.instructions.push(Instruction::NullCheck(
                source,
                real_source,
                "failed null check for conditional field".to_string(),
            ));
            real_source
        } else {
            source
        };

        match &*field.type_.borrow() {
            Type::Container(c) => {
                let buf_target = if let Some(length) = &c.length {
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
                if c.is_enum.get() {
                    let break_start = self.instructions.len();
                    for (name, child) in c.items.iter() {
                        let condition = self.encode_field_condition(child);
                        let start = self.instructions.len();
                        let unwrapped = self.alloc_register();

                        let subtype = child.type_.borrow();
                        match &*subtype {
                            Type::Container(c) => {

                                let mut unwrapped = vec![];
                                for (subname, subchild) in c.flatten_view() {
                                    if !matches!(&*subchild.type_.borrow(), Type::Container(_)) {
                                        let alloced = self.alloc_register();
                                        unwrapped.push((
                                            subname.clone(),
                                            alloced,
                                        ));
                                    }
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
                                self.encode_container_items(c, buf_target, &resolver, source, );
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
                    self.encode_container_items(c, buf_target, resolver, source);
                }

                if let Some(length) = &c.length {
                    match length {
                        Expression::FieldRef(f) if f.is_auto.get() => {
                            let type_ = f.type_.borrow();
                            let cast_type = match type_.resolved().as_ref() {
                                Type::Scalar(s) => *s,
                                Type::Foreign(f) => match f.obj.can_receive_auto() {
                                    Some(s) => s,
                                    None => unimplemented!("bad ffi type for auto field"),
                                },
                                _ => unimplemented!("bad type for auto field"),
                            };

                            let target = self.alloc_register();
                            self.instructions.push(Instruction::GetLen(
                                target,
                                buf_target.unwrap_buf(),
                                Some(cast_type),
                            ));
                            self.resolved_autos.insert(f.name.clone(), target);
                        }
                        _ => (),
                    }
                    self.instructions
                        .push(Instruction::EmitBuf(target, buf_target.unwrap_buf()));
                }
            }
            t => self.encode_type(target, resolver, source, t),
        }

        for (stream, owned_stream) in new_streams.iter().rev() {
            self.instructions.push(Instruction::EndStream(*stream));
            if let Some(owned_stream) = owned_stream {
                self.instructions.push(Instruction::Drop(*owned_stream));
            }
        }
    }

    pub fn encode_type(&mut self, target: Target, resolver: &Resolver, source: usize, type_: &Type) {
        match type_ {
            Type::Container(_) => unimplemented!(),
            Type::Array(c) => {
                let terminator = if c.length.expandable && c.length.value.is_some() {
                    let len = c.length.value.as_ref().cloned().unwrap();
                    let r = self.alloc_register();
                    self.instructions.push(Instruction::Eval(r, len));
                    Some(r)
                } else {
                    None
                };

                let mut len = if terminator.is_none() {
                    match &c.length.value {
                        Some(Expression::FieldRef(f)) if f.is_auto.get() => {
                            let type_ = f.type_.borrow();
                            let cast_type = match &*type_ {
                                Type::Scalar(s) => s,
                                _ => unimplemented!("bad type for auto field"),
                            };

                            let target = self.alloc_register();
                            self.instructions.push(Instruction::GetLen(
                                target,
                                source,
                                Some(*cast_type),
                            ));
                            self.resolved_autos.insert(f.name.clone(), target);
                            Some(target)
                        }
                        _ => None,
                    }
                } else {
                    None
                };

                if len.is_none() && !c.length.expandable {
                    len = {
                        let len = c.length.value.as_ref().cloned().unwrap();
                        let r = self.alloc_register();
                        self.instructions.push(Instruction::Eval(r, len));
                        Some(r)
                    };
                }

                if c.element.condition.borrow().is_none()
                    && c.element.transforms.borrow().len() == 0
                    && terminator.is_none()
                {
                    match &*c.element.type_.borrow() {
                        // todo: const-length type optimizations for container/array/foreign
                        Type::Container(_) | Type::Array(_) | Type::Foreign(_) | Type::Ref(_) => (),
                        Type::Enum(x) => {
                            self.instructions.push(Instruction::EncodePrimitiveArray(
                                target,
                                source,
                                PrimitiveType::Scalar(x.rep),
                                len,
                            ));
                            return;
                        }
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
                self.encode_field(target, resolver, new_source, &c.element);
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
            Type::Enum(e) => {
                self.instructions.push(Instruction::EncodeEnum(
                    PrimitiveType::Scalar(e.rep.clone()),
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
            Type::Ref(r) => {
                let mut args = vec![];
                for arg in r.arguments.iter() {
                    let r = self.alloc_register();
                    self.instructions.push(Instruction::Eval(r, arg.clone()));
                    args.push(r);
                }
                if let Type::Foreign(f) = &*r.target.type_.borrow() {
                    let arguments = f.obj.arguments();
                    for (expr, arg) in r.arguments.iter().zip(arguments.iter()) {
                        if arg.can_resolve_auto {
                            match expr {
                                Expression::FieldRef(f) if f.is_auto.get() => {
                                    let type_ = f.type_.borrow();
                                    let cast_type = match &*type_ {
                                        Type::Scalar(s) => s,
                                        _ => unimplemented!("bad type for auto field"),
                                    };

                                    let len_target = self.alloc_register();
                                    self.instructions.push(Instruction::GetLen(
                                        len_target,
                                        source,
                                        Some(*cast_type),
                                    ));
                                    self.resolved_autos.insert(f.name.clone(), len_target);
                                }
                                _ => (),
                            }
                        }
                    }
                    self.instructions.push(Instruction::EncodeForeign(
                        target,
                        source,
                        f.clone(),
                        args,
                    ));
                } else {
                    self.instructions
                        .push(Instruction::EncodeRef(target, source, args));
                }
            }
        }
    }
}
