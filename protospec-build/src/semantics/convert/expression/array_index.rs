use super::*;

impl Scope {
    pub(super) fn convert_array_index_expression(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::ArrayIndexExpression,
        expected_type: PartialType,
    ) -> AsgResult<ArrayIndexExpression> {
        Ok(ArrayIndexExpression {
            array: Box::new(Scope::convert_expr(
                self_,
                &expr.array,
                PartialType::Array(Some(Box::new(expected_type))),
            )?),
            index: Box::new(Scope::convert_expr(
                self_,
                &expr.index,
                Type::Scalar(ScalarType::U64.into()).into(),
            )?),
            span: expr.span,
        })
    }
}
