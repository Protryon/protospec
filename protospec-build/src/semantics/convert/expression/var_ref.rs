use super::*;

impl Scope {
    pub(super) fn convert_ref_expression(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::Ident,
        expected_type: PartialType,
    ) -> AsgResult<Expression> {
        let expression = if let Some(field) = Scope::resolve_field(self_, &expr.name) {
            Expression::FieldRef(field)
        } else if let Some(input) = Scope::resolve_input(self_, &expr.name) {
            Expression::InputRef(input)
        } else if let Some(cons) = self_.borrow().program.borrow().consts.get(&expr.name) {
            Expression::ConstRef(cons.clone())
        } else {
            return Err(AsgError::UnresolvedVar(expr.name.clone(), expr.span));
        };
        let type_ = expression.get_type().expect("ref missing type");
        if !expected_type.assignable_from(&type_) {
            return Err(AsgError::UnexpectedType(type_.to_string(), expected_type.to_string(), *expr.span()))
        }
        Ok(expression)
    }
}
