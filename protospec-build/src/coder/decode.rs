use super::*;
use crate::asg::*;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub enum Constructable {
    Struct {
        name: String,
        items: Vec<(String, usize)>,
    },
    Tuple(Vec<usize>),
    TaggedTuple {
        name: String,
        items: Vec<usize>,
    },
}

#[derive(Debug)]
pub enum Instruction {
    Eval(usize, Expression),
    Construct(usize, Constructable),
    // source, new_stream, len constraint
    Constrict(Target, usize, usize),
    WrapStream(Target, usize, Arc<Transform>, Vec<usize>), // stream, new stream, transformer, arguments
    ConditionalWrapStream(
        usize,
        Vec<Instruction>,
        Target,
        usize,
        Arc<Transform>,
        Vec<usize>,
    ), // condition, prelude, stream, new stream, transformer, arguments

    DecodeForeign(Target, usize, Arc<NamedForeignType>, Vec<usize>),
    DecodeRef(Target, usize, String, Vec<usize>),
    DecodeEnum(String, PrimitiveType, usize, Target),
    DecodePrimitive(Target, usize, PrimitiveType),
    DecodePrimitiveArray(Target, usize, PrimitiveType, Option<usize>),

    // register representing: internal stream, end index, terminator, output handle, inner
    Loop(
        Target,
        Option<usize>,
        Option<usize>,
        usize,
        Vec<Instruction>,
    ),
    LoopOutput(usize, usize), // output handle, item
    Conditional(usize, usize, usize, Vec<Instruction>), // target, interior_register, condition, if_true
}

#[derive(Debug)]
pub struct Context {
    pub register_count: usize,
    pub field_register_map: HashMap<String, usize>,
    pub instructions: Vec<Instruction>,
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
            field_register_map: HashMap::new(),
            register_count: 0,
        }
    }

    pub fn decode_field_top(&mut self, field: &Arc<Field>) -> usize {
        assert!(field.toplevel);
        let value = self.decode_field(Target::Direct, field).unwrap();
        match &*field.type_.borrow() {
            Type::Foreign(_) => (),
            Type::Container(_) => (),
            Type::Enum(_) => (),
            _ => {
                let extra_value = self.alloc_register();
                self.instructions.push(Instruction::Construct(
                    extra_value,
                    Constructable::TaggedTuple {
                        name: field.name.clone(),
                        items: vec![value],
                    },
                ));
                return extra_value;
            }
        }
        value
    }

    pub fn decode_field(&mut self, mut source: Target, field: &Arc<Field>) -> Option<usize> {
        let field_condition = if let Some(condition) = field.condition.borrow().as_ref() {
            let value = self.alloc_register();
            self.instructions
                .push(Instruction::Eval(value, condition.clone()));
            Some(value)
        } else {
            None
        };
        let start = self.instructions.len();
        let mut new_streams = vec![];

        for transform in field.transforms.borrow().iter().rev() {
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
        let emitted = match &*field.type_.borrow() {
            Type::Container(c) => {
                let buf_target = if let Some(length) = &c.length {
                    //todo: use limited stream
                    let len_register = self.alloc_register();
                    self.instructions
                        .push(Instruction::Eval(len_register, length.clone()));
                    let buf = self.alloc_register();
                    self.instructions
                        .push(Instruction::Constrict(source, buf, len_register));
                    Target::Stream(buf)
                } else {
                    source
                };
                for (name, child) in c.items.iter() {
                    let decoded = if matches!(&*child.type_.borrow(), Type::Container(_)) {
                        self.decode_field(buf_target, child)
                    } else {
                        self.decode_field(buf_target, child)
                    };
                    if let Some(decoded) = decoded {
                        self.field_register_map.insert(name.clone(), decoded);
                    }
                }
                if field.toplevel {
                    let emitted = self.alloc_register();
                    let mut items = vec![];
                    for (name, child) in c.flatten_view() {
                        if !matches!(&*child.type_.borrow(), Type::Container(_)) {
                            items.push((
                                name.clone(),
                                *self
                                    .field_register_map
                                    .get(&name)
                                    .expect("missing field in field_register_map"),
                            ));
                        }
                    }
                    self.instructions.push(Instruction::Construct(
                        emitted,
                        Constructable::Struct {
                            name: field.name.clone(),
                            items,
                        },
                    ));
                    Some(emitted)
                } else {
                    None
                }
            }
            _ => Some(self.decode_type(source, field)),
        };

        if let Some(field_condition) = field_condition {
            if emitted.is_none() {
                unimplemented!("cannot have interior containers with field conditional");
            }
            let target = self.alloc_register();
            let drained = self.instructions.drain(start..).collect();
            self.instructions.push(Instruction::Conditional(
                target,
                emitted.unwrap(),
                field_condition,
                drained,
            ));
            Some(target)
        } else {
            emitted
        }
    }

    pub fn decode_type(&mut self, source: Target, field: &Arc<Field>) -> usize {
        let output = self.alloc_register();
        match &*field.type_.borrow() {
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

                let len = if c.length.expandable {
                    None
                } else {
                    let len = c.length.value.as_ref().cloned().unwrap();
                    let r = self.alloc_register();
                    self.instructions.push(Instruction::Eval(r, len));
                    Some(r)
                };

                if c.element.condition.borrow().is_none()
                    && c.element.transforms.borrow().len() == 0
                    && terminator.is_none()
                {
                    match &*c.element.type_.borrow() {
                        // todo: const-length type optimizations for container/array/foreign
                        Type::Container(_) | Type::Array(_) | Type::Foreign(_) | Type::Ref(_) => (),
                        Type::Enum(x) => {
                            self.instructions.push(Instruction::DecodePrimitiveArray(
                                source,
                                output,
                                PrimitiveType::Scalar(x.rep),
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
                let item = self.decode_field(source, &c.element);
                if item.is_none() {
                    unimplemented!("cannot have inline container inside array");
                }
                self.instructions
                    .push(Instruction::LoopOutput(output, item.unwrap()));
                let drained = self.instructions.drain(current_pos..).collect();
                self.instructions
                    .push(Instruction::Loop(source, len, terminator, output, drained));
                output
            }
            Type::Enum(e) => {
                self.instructions.push(Instruction::DecodeEnum(
                    field.name.clone(),
                    PrimitiveType::Scalar(e.rep.clone()),
                    output,
                    source,
                ));
                output
            }
            Type::Scalar(s) => {
                self.instructions.push(Instruction::DecodePrimitive(
                    source,
                    output,
                    PrimitiveType::Scalar(*s),
                ));
                output
            }
            Type::F32 => {
                self.instructions.push(Instruction::DecodePrimitive(
                    source,
                    output,
                    PrimitiveType::F32,
                ));
                output
            }
            Type::F64 => {
                self.instructions.push(Instruction::DecodePrimitive(
                    source,
                    output,
                    PrimitiveType::F64,
                ));
                output
            }
            Type::Bool => {
                self.instructions.push(Instruction::DecodePrimitive(
                    source,
                    output,
                    PrimitiveType::Bool,
                ));
                output
            }
            Type::Foreign(f) => {
                self.instructions.push(Instruction::DecodeForeign(
                    source,
                    output,
                    f.clone(),
                    vec![],
                ));
                output
            }
            Type::Ref(r) => {
                let mut args = vec![];
                for arg in r.arguments.iter() {
                    let r = self.alloc_register();
                    self.instructions.push(Instruction::Eval(r, arg.clone()));
                    args.push(r);
                }
                if let Type::Foreign(f) = &*r.target.type_.borrow() {
                    self.instructions.push(Instruction::DecodeForeign(
                        source,
                        output,
                        f.clone(),
                        args,
                    ));
                } else {
                    self.instructions.push(Instruction::DecodeRef(
                        source,
                        output,
                        r.target.name.clone(),
                        args,
                    ));
                }
                output
            }
        }
    }
}
