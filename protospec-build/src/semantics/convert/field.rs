use super::*;

impl Scope {
    pub fn convert_ast_field_arguments(
        self_: &Arc<RefCell<Scope>>,
        into: &Arc<Field>,
        ast_arguments: Option<&[ast::TypeArgument]>,
    ) -> AsgResult<Arc<RefCell<Scope>>> {
        let sub_scope = Arc::new(RefCell::new(Scope {
            parent_scope: Some(self_.clone()),
            program: self_.borrow().program.clone(),
            declared_fields: IndexMap::new(),
            declared_inputs: IndexMap::new(),
        }));

        let mut arguments = vec![];
        if let Some(ast_arguments) = ast_arguments {
            for argument in ast_arguments {
                let target_type = Scope::convert_ast_type(&sub_scope, &argument.type_.raw_type, false)?;
                sub_scope.borrow_mut().declared_inputs.insert(
                    argument.name.name.clone(),
                    Arc::new(Input {
                        name: argument.name.name.clone(),
                        type_: target_type.clone(),
                    }),
                );
                arguments.push(TypeArgument {
                    name: argument.name.name.clone(),
                    type_: target_type.clone(),
                    default_value: argument
                        .default_value
                        .as_ref()
                        .map(|expr| Scope::convert_expr(&sub_scope, expr, target_type.into()))
                        .transpose()?,
                    can_resolve_auto: false,
                });
            }
        }

        into.arguments.replace(arguments);
        Ok(sub_scope)
    }

    pub fn convert_ast_field(
        sub_scope: &Arc<RefCell<Scope>>,
        field: &ast::Field,
        into: &Arc<Field>,
    ) -> AsgResult<()> {
        let condition = if let Some(condition) = &field.condition {
            Some(Scope::convert_expr(
                &sub_scope,
                &**condition,
                PartialType::Type(Type::Bool),
            )?)
        } else {
            None
        };

        let asg_type = Scope::convert_ast_type(&sub_scope, &field.type_.raw_type, into.toplevel)?;

        let mut transforms = vec![];
        for ast::Transform {
            name,
            conditional,
            arguments,
            span,
        } in field.transforms.iter()
        {
            let def_transform = if let Some(def_transform) =
                sub_scope.borrow().program.borrow().transforms.get(&name.name)
            {
                def_transform.clone()
            } else {
                return Err(AsgError::UnresolvedTransform(name.name.clone(), name.span));
            };
            let arguments = Self::convert_ffi_arguments(
                sub_scope,
                &*def_transform.name,
                *span,
                &arguments[..],
                &def_transform.arguments[..],
            )?;

            transforms.push(TypeTransform {
                transform: def_transform,
                condition: if let Some(conditional) = conditional {
                    Some(Scope::convert_expr(
                        sub_scope,
                        &**conditional,
                        PartialType::Type(Type::Bool),
                    )?)
                } else {
                    None
                },
                arguments,
            })
        }
        let mut is_auto = false;
        for flag in field.flags.iter() {
            match &*flag.name {
                "auto" => {
                    match asg_type.resolved().as_ref() {
                        Type::Scalar(_) => (),
                        Type::Foreign(f) if f.obj.can_receive_auto().is_some() => (),
                        other => return Err(AsgError::TypeNotAutoCompatible(other.to_string(), field.type_.span)),
                    }
                    is_auto = true;
                }
                x => return Err(AsgError::InvalidFlag(x.to_string(), flag.span)),
            }
        }

        into.type_.replace(asg_type);
        into.condition.replace(condition);
        into.transforms.replace(transforms);
        into.is_auto.replace(is_auto);

        Ok(())
    }
}