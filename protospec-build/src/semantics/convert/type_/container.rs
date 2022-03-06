use crate::ContainerItem;

use super::*;

impl Scope {
    pub(super) fn convert_container_type(
        self_: &Arc<RefCell<Scope>>,
        type_: &ast::Container,
        purpose: TypePurpose,
    ) -> AsgResult<Type> {
        let length = type_
            .length
            .as_ref()
            .map(|x| {
                Scope::convert_expr(self_, &**x, PartialType::Scalar(PartialScalarType::Defaults(ScalarType::U64)))
            })
            .transpose()?;

        let mut is_enum = false;
        for flag in &type_.flags {
            match &*flag.name {
                "tagged_enum" => is_enum = true,
                x => return Err(AsgError::InvalidFlag(x.to_string(), flag.span)),
            }
        }

        if is_enum && !matches!(purpose, TypePurpose::TypeDefinition(_)) {
            return Err(AsgError::MustBeToplevel(type_.span));
        }
        
        let mut items: IndexMap<String, Arc<Field>> = IndexMap::new();
        let sub_scope = Arc::new(RefCell::new(Scope {
            parent_scope: Some(self_.clone()),
            program: self_.borrow().program.clone(),
            declared_fields: IndexMap::new(),
            declared_inputs: IndexMap::new(),
        }));

        let mut had_unconditional_field = false;

        let mut field_scopes = vec![];

        let mut pad_count = 0;
        for item in type_.items.iter() {
            match item {
                ContainerItem::Field(name, ast_field) => {
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
                        calculated: RefCell::new(None),
                        condition: RefCell::new(None),
                        transforms: RefCell::new(vec![]),
                        span: ast_field.span,
                        toplevel: false,
                        arguments: RefCell::new(vec![]),
                        is_maybe_cyclical: Cell::new(false),
                        is_pad: Cell::new(false),
                    });
        
                    {
                        let sub_scope = Scope::convert_ast_field_arguments(&sub_scope, &field_out, None)?;
                        Scope::convert_ast_field_mid(&sub_scope, ast_field, &field_out)?;
                        field_scopes.push((field_out.clone(), sub_scope, ast_field.clone()));
                    }
        
                    if had_unconditional_field && is_enum {
                        return Err(AsgError::EnumContainerFieldAfterUnconditional(ast_field.span))
                    }
                    if ast_field.condition.is_none() {
                        had_unconditional_field = true;
                    }
        
                    sub_scope
                        .borrow_mut()
                        .declared_fields
                        .insert(name.name.clone(), field_out.clone());
                    items.insert(name.name.clone(), field_out);
                }
                ContainerItem::Pad(expr) => {
                    if is_enum {
                        return Err(AsgError::EnumContainerPad(*expr.span()));
                    }
                    let name = format!("_pad{}", pad_count);
                    pad_count += 1;

                    let len = Scope::convert_expr(&sub_scope, expr, PartialType::Scalar(PartialScalarType::Some(ScalarType::U64)))?;

                    let field_out = Arc::new(Field {
                        name: name.clone(),
                        type_: RefCell::new(Type::Array(Box::new(ArrayType {
                            element: Box::new(Type::Scalar(ScalarType::U8)),
                            length: LengthConstraint {
                                expandable: false,
                                value: Some(len),
                            }
                        }))),
                        calculated: RefCell::new(None),
                        condition: RefCell::new(None),
                        transforms: RefCell::new(vec![]),
                        span: *expr.span(),
                        toplevel: false,
                        arguments: RefCell::new(vec![]),
                        is_maybe_cyclical: Cell::new(false),
                        is_pad: Cell::new(true),
                    });
        
                    items.insert(name.clone(), field_out);
                }
            }
        }
        for (out_field, sub_scope, ast_field) in field_scopes {
            Scope::convert_ast_field_end(&sub_scope, &ast_field, &out_field)?;
        }

        Ok(Type::Container(Box::new(ContainerType {
            length,
            items,
            is_enum: Cell::new(is_enum),
        })))
    }
}
