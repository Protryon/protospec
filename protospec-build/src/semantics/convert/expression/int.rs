use super::*;

impl Scope {
    pub(super) fn convert_int_expression(
        _self_: &Arc<RefCell<Scope>>,
        expr: &ast::Int,
        expected_type: PartialType,
    ) -> AsgResult<Int> {
        match (&expected_type, &expr.type_) {
            (x, Some(y)) if x.assignable_from(&Type::Scalar(*y)) => (),
            (PartialType::Scalar(Some(_)), None) => (),
            (PartialType::Scalar(None), Some(_)) => (),
            (PartialType::Any, Some(_)) => (),
            (x, Some(y)) => {
                return Err(AsgError::UnexpectedType(
                    y.to_string(),
                    x.to_string(),
                    expr.span,
                ));
            }
            (x, _) => {
                return Err(AsgError::UnexpectedType(
                    "integer".to_string(),
                    x.to_string(),
                    expr.span,
                ));
            }
        }
        let type_ = match (&expected_type, &expr.type_) {
            (_, Some(s)) => *s,
            (PartialType::Scalar(Some(s)), _) => *s,
            _ => unimplemented!(),
        };
        Ok(Int {
            value: ConstInt::parse(type_, &expr.value, expr.span)?,
            type_,
            span: expr.span,
        })
    }
}
