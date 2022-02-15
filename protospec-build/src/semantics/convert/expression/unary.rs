use super::*;

impl Scope {
    pub(super) fn convert_unary_expression(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::UnaryExpression,
        expected_type: PartialType,
    ) -> AsgResult<UnaryExpression> {
        let inner = Box::new(Scope::convert_expr(
            self_,
            &expr.inner,
            expected_type.clone(),
        )?);
        match expr.op {
            ast::UnaryOp::Not => {
                if !expected_type.assignable_from(&Type::Bool) {
                    return Err(AsgError::UnexpectedType(
                        "bool".to_string(),
                        expected_type.to_string(),
                        expr.span,
                    ));
                }
            }
            ast::UnaryOp::Negate | ast::UnaryOp::BitNot => {
                if let Some(inner_type) = inner.get_type() {
                    if !PartialType::Scalar(PartialScalarType::None).assignable_from(&inner_type) {
                        return Err(AsgError::UnexpectedType(
                            inner_type.to_string(),
                            "integer".to_string(),
                            expr.span,
                        ));
                    }
                    if !expected_type.assignable_from(&inner_type) {
                        return Err(AsgError::UnexpectedType(
                            inner_type.to_string(),
                            expected_type.to_string(),
                            expr.span,
                        ));
                    }
                    if expr.op == ast::UnaryOp::Negate {
                        match inner_type {
                            Type::Scalar(s) if !s.is_signed() => {
                                return Err(AsgError::UnexpectedType(
                                    inner_type.to_string(),
                                    "signed integer".to_string(),
                                    expr.span,
                                ));
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
        Ok(UnaryExpression {
            op: expr.op.clone(),
            inner: Box::new(Scope::convert_expr(self_, &expr.inner, expected_type)?),
            span: expr.span,
        })
    }
}
