use super::*;

impl Scope {
    pub(super) fn convert_call_expression(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::CallExpression,
        expected_type: PartialType,
    ) -> AsgResult<CallExpression> {
        let scope = self_.borrow();

        let function = scope
            .program
            .borrow()
            .functions
            .get(&*expr.function.name)
            .ok_or_else(|| {
                AsgError::UnresolvedFunction(expr.function.name.clone(), expr.function.span)
            })?
            .clone();
        
        let return_type = function.inner.return_type();
        if !expected_type.assignable_from(&return_type) {
            return Err(AsgError::UnexpectedType(return_type.to_string(), expected_type.to_string(), expr.span));
        }

        let arguments = Self::convert_ffi_arguments(
            self_,
            &*function.name,
            expr.span,
            &expr.arguments[..],
            &function.arguments[..],
        )?;

        Ok(CallExpression {
            function,
            arguments,
            span: expr.span,
        })
    }
}
