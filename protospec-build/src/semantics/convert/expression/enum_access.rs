use super::*;

impl Scope {
    pub(super) fn convert_enum_access_expression(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::EnumAccessExpression,
        _expected_type: PartialType,
    ) -> AsgResult<EnumAccessExpression> {
        let field = match self_.borrow().program.borrow().types.get(&expr.name.name) {
            Some(x) => x.clone(),
            None => {
                return Err(AsgError::UnresolvedType(
                    expr.name.name.clone(),
                    expr.name.span,
                ))
            }
        };
        let type_ = field.type_.borrow();
        let variant = match &*type_ {
            Type::Enum(e) => {
                let variant =
                    e.items
                        .get(&expr.variant.name)
                        .ok_or(AsgError::UnresolvedEnumVariant(
                            field.name.clone(),
                            expr.variant.name.clone(),
                            expr.variant.span,
                        ))?;
                match variant {
                    EnumValue::Value(value) => value.clone(),
                    EnumValue::Default => {
                        return Err(AsgError::ReferencedDefaultEnumVariant(
                            field.name.clone(),
                            expr.variant.name.clone(),
                            expr.variant.span,
                        ))
                    }
                }
            }
            Type::Bitfield(e) => e
                .items
                .get(&expr.variant.name)
                .ok_or(AsgError::UnresolvedBitfieldVariant(
                    field.name.clone(),
                    expr.variant.name.clone(),
                    expr.variant.span,
                ))?
                .clone(),
            _ => {
                return Err(AsgError::UnexpectedType(
                    field.type_.borrow().to_string(),
                    "enum or bitfield".to_string(),
                    expr.span,
                ));
            }
        };
        drop(type_);

        Ok(EnumAccessExpression {
            enum_field: field,
            variant,
            span: expr.span,
        })
    }
}
