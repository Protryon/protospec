use super::*;

impl Scope {
    pub(super) fn convert_binary_expression(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::BinaryExpression,
        expected_type: PartialType,
    ) -> AsgResult<BinaryExpression> {
        use ast::BinaryOp::*;
        match expr.op {
            Lt | Gt | Lte | Gte | Eq | Ne | Or | And => {
                if !expected_type.assignable_from(&Type::Bool) {
                    return Err(AsgError::UnexpectedType(
                        "bool".to_string(),
                        expected_type.to_string(),
                        expr.span,
                    ));
                }
            }
            _ => ()
        }
        let init_expected_type = match expr.op {
            Lt | Gt | Lte | Gte => PartialType::Scalar(PartialScalarType::None),
            Eq | Ne => PartialType::Any,
            Or | And => PartialType::Type(Type::Bool),
            _ => expected_type.clone(),
        };
        let mut left = Scope::convert_expr(self_, &expr.left, init_expected_type.clone());
        let right =
            if let Some(left_type) = left.as_ref().map(|x| x.get_type()).ok().flatten() {
                Scope::convert_expr(self_, &expr.right, left_type.into())?
            } else {
                let right = Scope::convert_expr(self_, &expr.right, init_expected_type)?;
                if let Some(right_type) = right.get_type() {
                    left = Ok(Scope::convert_expr(self_, &expr.left, right_type.into())?);
                    if left.as_ref().unwrap().get_type().is_none() {
                        return Err(AsgError::UninferredType(*expr.left.span()));
                    }
                } else {
                    return Err(AsgError::UninferredType(expr.span));
                }
                right
            };
        Ok(BinaryExpression {
            op: expr.op.clone(),
            left: Box::new(left.unwrap()),
            right: Box::new(right),
            span: expr.span,
        })
    }
}
