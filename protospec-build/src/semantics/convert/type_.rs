use super::*;

impl Scope {

    pub fn convert_ast_type(self_: &Arc<RefCell<Scope>>, typ: &ast::RawType, toplevel: bool) -> AsgResult<Type> {
        Ok(match typ {
            ast::RawType::Container(value) => {
                let length = value
                    .length
                    .as_ref()
                    .map(|x| {
                        Scope::convert_expr(self_, &**x, PartialType::Scalar(Some(ScalarType::U64)))
                    })
                    .transpose()?;

                let mut is_enum = false;
                for flag in &value.flags {
                    match &*flag.name {
                        "tagged_enum" => is_enum = true,
                        x => return Err(AsgError::InvalidFlag(x.to_string(), flag.span)),
                    }
                }

                if !toplevel && is_enum {
                    return Err(AsgError::EnumContainerMustBeToplevel(value.span));
                }
                
                let mut items: IndexMap<String, Arc<Field>> = IndexMap::new();
                let sub_scope = Arc::new(RefCell::new(Scope {
                    parent_scope: Some(self_.clone()),
                    program: self_.borrow().program.clone(),
                    declared_fields: IndexMap::new(),
                    declared_inputs: IndexMap::new(),
                }));

                let mut had_unconditional_field = false;

                for (name, typ) in value.items.iter() {
                    if let Some(defined) = items.get(&name.name) {
                        return Err(AsgError::ContainerFieldRedefinition(
                            name.name.clone(),
                            name.span,
                            defined.span,
                        ));
                    }
                    let field_out = Arc::new(Field {
                        name: name.name.clone(),
                        type_: RefCell::new(Type::Bool),
                        condition: RefCell::new(None),
                        transforms: RefCell::new(vec![]),
                        span: typ.span,
                        toplevel: false,
                        arguments: RefCell::new(vec![]),
                        is_auto: Cell::new(false),
                        is_maybe_cyclical: Cell::new(false),
                    });

                    Scope::convert_ast_field(&sub_scope, typ, &field_out, None)?;

                    if had_unconditional_field && is_enum {
                        return Err(AsgError::EnumContainerFieldAfterUnconditional(typ.span))
                    }
                    if field_out.condition.borrow().is_none() {
                        had_unconditional_field = true;
                    }

                    sub_scope
                        .borrow_mut()
                        .declared_fields
                        .insert(name.name.clone(), field_out.clone());
                    items.insert(name.name.clone(), field_out);
                }

                Type::Container(Box::new(ContainerType {
                    length,
                    items,
                    is_enum: Cell::new(is_enum),
                }))
            }
            ast::RawType::Enum(value) => {
                let mut items: IndexMap<String, Arc<Const>> = IndexMap::new();
                let mut last_defined_item = None::<Arc<Const>>;
                let mut undefined_counter = 0usize;
                for (name, item) in value.items.iter() {
                    if let Some(prior) = items.get(&name.name) {
                        return Err(AsgError::EnumVariantRedefinition(
                            name.name.clone(),
                            name.span,
                            prior.span,
                        ));
                    }
                    //todo: static eval here
                    let cons = Arc::new(Const {
                        name: name.name.clone(),
                        span: value.span,
                        type_: Type::Scalar(value.rep),
                        value: match item {
                            Some(expr) => Scope::convert_expr(
                                self_,
                                &**expr,
                                PartialType::Scalar(Some(value.rep)),
                            )?,
                            None => Expression::Binary(BinaryExpression {
                                op: crate::BinaryOp::Add,
                                left: Box::new(Expression::ConstRef(
                                    last_defined_item.as_ref().unwrap().clone(),
                                )),
                                right: Box::new(Expression::Int(Int {
                                    value: ConstInt::parse(
                                        value.rep,
                                        &*format!("{}", undefined_counter),
                                        name.span,
                                    )?,
                                    type_: value.rep,
                                    span: name.span,
                                })),
                                span: value.span,
                            }),
                        },
                    });
                    if item.is_some() {
                        last_defined_item = Some(cons.clone());
                        undefined_counter = 1;
                    } else {
                        undefined_counter += 1;
                    }
                    items.insert(name.name.clone(), cons);
                }
                Type::Enum(EnumType {
                    rep: value.rep,
                    items,
                })
            }
            ast::RawType::Scalar(value) => Type::Scalar(value.clone()),
            ast::RawType::Array(value) => {
                let length = Scope::convert_length(self_, &value.length)?;
                let element = Scope::convert_ast_type(self_, &value.element.type_.raw_type, false)?;
                match &element {
                    Type::Container(_) | Type::Enum(_) => {
                        return Err(AsgError::InlineRepetition(value.span));
                    }
                    _ => (),
                }
                let field = Arc::new(Field {
                    name: "$array_field".to_string(),
                    type_: RefCell::new(element),
                    arguments: RefCell::new(vec![]),
                    condition: RefCell::new(None),
                    transforms: RefCell::new(vec![]),
                    span: value.span,
                    toplevel: false,
                    is_auto: Cell::new(false),
                    is_maybe_cyclical: Cell::new(false),
                });

                Type::Array(Box::new(ArrayType {
                    element: field,
                    length,
                }))
            }
            ast::RawType::F32 => Type::F32,
            ast::RawType::F64 => Type::F64,
            ast::RawType::Bool => Type::Bool,
            ast::RawType::Ref(call) => {
                if let Some(target) = self_.borrow().program.borrow().types.get(&call.name.name) {
                    let target_args = target.arguments.borrow();
                    let min_arg_count = target_args
                        .iter()
                        .filter(|x| x.default_value.is_some())
                        .count();
                    // optionals MUST be at the end
                    if min_arg_count < call.arguments.len()
                        && target_args[target_args.len() - min_arg_count..]
                            .iter()
                            .any(|x| x.default_value.is_some())
                    {
                        return Err(AsgError::InvalidTypeArgumentOrder(call.span));
                    }
                    if call.arguments.len() < min_arg_count
                        || call.arguments.len() > target_args.len()
                    {
                        return Err(AsgError::InvalidTypeArgumentCount(
                            min_arg_count,
                            target_args.len(),
                            call.arguments.len(),
                            call.span,
                        ));
                    }
                    let arguments = call
                        .arguments
                        .iter()
                        .zip(target_args.iter())
                        .map(|(expr, argument)| {
                            Scope::convert_expr(self_, expr, argument.type_.clone().into())
                        })
                        .collect::<AsgResult<Vec<Expression>>>()?;

                    Type::Ref(TypeRef {
                        target: target.clone(),
                        arguments,
                    })
                } else {
                    return Err(AsgError::UnresolvedType(
                        call.name.name.clone(),
                        call.name.span,
                    ));
                }
            }
        })
    }
}