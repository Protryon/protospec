use super::*;

impl Scope {
    pub(super) fn convert_str_expression(
        _self_: &Arc<RefCell<Scope>>,
        expr: &ast::Str,
        expected_type: PartialType,
    ) -> AsgResult<Expression> {
        let out = Expression::Str(expr.clone());
        let out_type = out.get_type().expect("untyped string");
        if !expected_type.assignable_from(&out_type) {
            return Err(AsgError::UnexpectedType(
                out_type.to_string(),
                expected_type.to_string(),
                expr.span,
            ));
        }
        Ok(out)
    }
}
