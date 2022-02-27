use super::*;

impl Scope {
    pub(super) fn convert_cast_expression(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::CastExpression,
        _expected_type: PartialType,
    ) -> AsgResult<CastExpression> {
        if !expr.type_.raw_type.is_inlinable() {
            return Err(AsgError::CastTypeDefinition(expr.span));
        }
        let target = Scope::convert_ast_type(self_, &expr.type_.raw_type, TypePurpose::Expression)?;

        let inner = Box::new(Scope::convert_expr(self_, &expr.inner, PartialType::Any)?);
        if let Some(inner_type) = inner.get_type() {
            if !inner_type.can_cast_to(&target) {
                return Err(AsgError::IllegalCast(
                    inner_type.to_string(),
                    target.to_string(),
                    expr.span,
                ));
            }
        } else {
            return Err(AsgError::UninferredType(*expr.inner.span()));
        }

        Ok(CastExpression {
            type_: target,
            inner,
            span: expr.span,
        })
    }
}
