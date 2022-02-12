use super::*;

impl Scope {
    pub fn convert_ffi_arguments(
        self_: &Arc<RefCell<Scope>>,
        name: &str,
        span: Span,
        arguments: &[ast::Expression],
        type_arguments: &[FFIArgument],
    ) -> AsgResult<Vec<Expression>> {
        let min_arg_count = type_arguments.iter().filter(|x| !x.optional).count();
        // optionals MUST be at the end
        if min_arg_count < arguments.len()
            && type_arguments[type_arguments.len() - min_arg_count..]
                .iter()
                .any(|x| !x.optional)
        {
            unimplemented!("cannot have ffi transform with required arguments after optional arguments for '{}'", name);
        }
        if arguments.len() < min_arg_count || arguments.len() > type_arguments.len() {
            return Err(AsgError::InvalidFFIArgumentCount(
                min_arg_count,
                type_arguments.len(),
                arguments.len(),
                span,
            ));
        }
        let arguments = arguments
            .iter()
            .zip(type_arguments.iter())
            .map(|(expr, argument)| {
                Scope::convert_expr(
                    self_,
                    expr,
                    argument
                        .type_
                        .clone()
                        .map(|x| x.into())
                        .unwrap_or_else(|| PartialType::Any),
                )
            })
            .collect::<AsgResult<Vec<Expression>>>()?;
        Ok(arguments)
    }
}