use super::*;

impl Scope {
    pub(super) fn convert_bool_expression(
        _self_: &Arc<RefCell<Scope>>,
        expr: &ast::Bool,
        _expected_type: PartialType,
    ) -> AsgResult<bool> {
        Ok(expr.value)
    }
}
