use super::*;

impl Scope {
    pub(super) fn convert_type_ref_type(
        self_: &Arc<RefCell<Scope>>,
        type_: &ast::TypeRef,
    ) -> AsgResult<Type> {
        if let Some(target) = self_.borrow().program.borrow().types.get(&type_.name.name) {
            let target_args = target.arguments.borrow();
            let min_arg_count = target_args
                .iter()
                .filter(|x| x.default_value.is_none())
                .count();
            // optionals MUST be at the end
            if min_arg_count < type_.arguments.len()
                && target_args[target_args.len() - min_arg_count..]
                    .iter()
                    .any(|x| x.default_value.is_some())
            {
                return Err(AsgError::InvalidTypeArgumentOrder(type_.span));
            }
            if type_.arguments.len() < min_arg_count
                || type_.arguments.len() > target_args.len()
            {
                return Err(AsgError::InvalidTypeArgumentCount(
                    min_arg_count,
                    target_args.len(),
                    type_.arguments.len(),
                    type_.span,
                ));
            }
            let arguments = type_
                .arguments
                .iter()
                .zip(target_args.iter())
                .map(|(expr, argument)| {
                    Scope::convert_expr(self_, expr, argument.type_.clone().into())
                })
                .collect::<AsgResult<Vec<Expression>>>()?;

            Ok(Type::Ref(TypeRef {
                target: target.clone(),
                arguments,
            }))
        } else {
            Err(AsgError::UnresolvedType(
                type_.name.name.clone(),
                type_.name.span,
            ))
        }
    }
}
