use super::*;

impl Scope {
    pub(super) fn convert_bool_expression(
        _self_: &Arc<RefCell<Scope>>,
        expr: &ast::Bool,
        expected_type: PartialType,
    ) -> AsgResult<bool> {
        match &expected_type {
            PartialType::Type(Type::Bool) => (),
            x => {
                return Err(AsgError::UnexpectedType(
                    "bool".to_string(),
                    x.to_string(),
                    expr.span,
                ));
            }
        }
        Ok(expr.value)
    }
}
