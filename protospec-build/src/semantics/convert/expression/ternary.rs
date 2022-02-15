use super::*;

impl Scope {
    pub(super) fn convert_ternary_expression(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::TernaryExpression,
        expected_type: PartialType,
    ) -> AsgResult<TernaryExpression> {
        let condition = Scope::convert_expr(self_, &expr.condition, Type::Bool.into())?;
        let if_true = Scope::convert_expr(self_, &expr.if_true, expected_type.clone())?;
        let right_type = match expected_type {
            PartialType::Any => if_true
                .get_type()
                .map(|x| x.into())
                .ok_or(AsgError::UninferredType(*expr.if_true.span()))?,
            x => x,
        };
        let if_false = Scope::convert_expr(self_, &expr.if_false, right_type)?;
        Ok(TernaryExpression {
            condition: Box::new(condition),
            if_true: Box::new(if_true),
            if_false: Box::new(if_false),
            span: expr.span,
        })
    }
}
