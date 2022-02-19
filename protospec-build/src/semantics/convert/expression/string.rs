use super::*;

impl Scope {
    pub(super) fn convert_str_expression(
        _self_: &Arc<RefCell<Scope>>,
        expr: &ast::Str,
        _expected_type: PartialType,
    ) -> AsgResult<Expression> {
        Ok(Expression::Str(expr.clone()))
    }
}
