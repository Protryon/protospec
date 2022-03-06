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
                let target_type = Scope::convert_ast_type(
                    &sub_scope,
                    &argument.type_.raw_type,
                    TypePurpose::Expression,
                )?;
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

    pub fn convert_ast_field_mid(
        sub_scope: &Arc<RefCell<Scope>>,
        field: &ast::Field,
        into: &Arc<Field>,
    ) -> AsgResult<()> {
        let purpose = if into.toplevel {
            TypePurpose::TypeDefinition(into.name.clone())
        } else {
            TypePurpose::FieldInterior
        };

        let asg_type = Scope::convert_ast_type(&sub_scope, &field.type_.raw_type, purpose)?;

        let condition = if let Some(condition) = &field.condition {
            Some(Scope::convert_expr(
                &sub_scope,
                &**condition,
                PartialType::Type(Type::Bool),
            )?)
        } else {
            None
        };

        let mut transforms = vec![];
        for ast::Transform {
            name,
            conditional,
            arguments,
            span,
        } in field.transforms.iter()
        {
            let def_transform = if let Some(def_transform) = sub_scope
                .borrow()
                .program
                .borrow()
                .transforms
                .get(&name.name)
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
        for flag in field.flags.iter() {
            match &*flag.name {
                x => return Err(AsgError::InvalidFlag(x.to_string(), flag.span)),
            }
        }

        // if !into.toplevel && condition.is_some() {
        //     if let Type::Container(type_) = &asg_type {
        //         for (_, child) in type_.flatten_view() {
        //             let mut child_condition = child.condition.borrow_mut();
        //             if child_condition.is_none() {
        //                 *child_condition = condition.clone();
        //             } else {
        //                 *child_condition = Some(Expression::Binary(BinaryExpression {
        //                     op: BinaryOp::And,
        //                     left: Box::new(condition.clone().unwrap()),
        //                     right: Box::new(child_condition.clone().unwrap()),
        //                     span: Span::default(),
        //                 }));
        //             }
        //         }
        //         condition = None;
        //     }
        // }

        into.type_.replace(asg_type);
        into.condition.replace(condition);
        into.transforms.replace(transforms);

        Ok(())
    }

    pub fn convert_ast_field_end(
        sub_scope: &Arc<RefCell<Scope>>,
        field: &ast::Field,
        into: &Arc<Field>,
    ) -> AsgResult<()> {
        let field_type = into.type_.borrow();

        let calculated = if let Some(calculated) = &field.calculated {
            Some(Scope::convert_expr(
                &sub_scope,
                &**calculated,
                field_type.clone().into(),
            )?)
        } else {
            None
        };

        into.calculated.replace(calculated);

        Ok(())
    }
}
