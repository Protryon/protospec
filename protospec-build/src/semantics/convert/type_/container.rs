use super::*;

impl Scope {
    pub(super) fn convert_container_type(
        self_: &Arc<RefCell<Scope>>,
        type_: &ast::Container,
        toplevel: bool,
    ) -> AsgResult<Type> {
        let length = type_
            .length
            .as_ref()
            .map(|x| {
                Scope::convert_expr(self_, &**x, PartialType::Scalar(None))
            })
            .transpose()?;

        let mut is_enum = false;
        for flag in &type_.flags {
            match &*flag.name {
                "tagged_enum" => is_enum = true,
                x => return Err(AsgError::InvalidFlag(x.to_string(), flag.span)),
            }
        }

        if !toplevel && is_enum {
            return Err(AsgError::EnumContainerMustBeToplevel(type_.span));
        }
        
        let mut items: IndexMap<String, Arc<Field>> = IndexMap::new();
        let sub_scope = Arc::new(RefCell::new(Scope {
            parent_scope: Some(self_.clone()),
            program: self_.borrow().program.clone(),
            declared_fields: IndexMap::new(),
            declared_inputs: IndexMap::new(),
        }));

        let mut had_unconditional_field = false;

        for (name, typ) in type_.items.iter() {
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

        Ok(Type::Container(Box::new(ContainerType {
            length,
            items,
            is_enum: Cell::new(is_enum),
        })))
    }
}
