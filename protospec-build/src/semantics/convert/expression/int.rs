use super::*;

impl Scope {
    pub(super) fn convert_int_expression(
        _self_: &Arc<RefCell<Scope>>,
        expr: &ast::Int,
        expected_type: PartialType,
    ) -> AsgResult<Int> {
        let type_ = match (&expected_type, &expr.type_) {
            (_, Some(s)) => *s,
            (PartialType::Scalar(PartialScalarType::Some(s)), _) => *s,
            (PartialType::Scalar(PartialScalarType::Defaults(s)), _) => *s,
            (x, _) => return Err(AsgError::UnexpectedType(
                "integer".to_string(),
                x.to_string(),
                expr.span,
            )),
        };
        Ok(Int {
            value: ConstInt::parse(type_, &expr.value, expr.span)?,
            type_,
            span: expr.span,
        })
    }
}
