use super::*;

impl Scope {
    pub(super) fn convert_enum_access_expression(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::EnumAccessExpression,
        expected_type: PartialType,
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
        let variant = match &*field.type_.borrow() {
            Type::Enum(e) => e
                .items
                .get(&expr.variant.name)
                .ok_or(AsgError::UnresolvedEnumVariant(
                    field.name.clone(),
                    expr.variant.name.clone(),
                    expr.variant.span,
                ))?
                .clone(),
            _ => {
                return Err(AsgError::UnexpectedType(
                    field.type_.borrow().to_string(),
                    "enum".to_string(),
                    expr.span,
                ));
            }
        };
        if !expected_type.assignable_from(&variant.type_) {
            return Err(AsgError::UnexpectedType(
                variant.type_.to_string(),
                expected_type.to_string(),
                expr.span,
            ));
        }
        Ok(EnumAccessExpression {
            enum_field: field,
            variant,
            span: expr.span,
        })
    }
}
