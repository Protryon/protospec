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
            Type::Enum(e) => &e.items,
            Type::Bitfield(e) => &e.items,
            _ => {
                return Err(AsgError::UnexpectedType(
                    field.type_.borrow().to_string(),
                    "enum or bitfield".to_string(),
                    expr.span,
                ));
            }
        };
        let variant = variant
            .get(&expr.variant.name)
            .ok_or(AsgError::UnresolvedEnumVariant(
                field.name.clone(),
                expr.variant.name.clone(),
                expr.variant.span,
            ))?
            .clone();
        drop(type_);

        Ok(EnumAccessExpression {
            enum_field: field,
            variant,
            span: expr.span,
        })
    }
}
