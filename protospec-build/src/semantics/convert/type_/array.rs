use super::*;

impl Scope {
    pub(super) fn convert_array_type(
        self_: &Arc<RefCell<Scope>>,
        type_: &ast::Array,
    ) -> AsgResult<Type> {
        let length = Scope::convert_length(self_, &type_.length)?;
        let element = Scope::convert_ast_type(
            self_,
            &type_.interior_type.raw_type,
            TypePurpose::ArrayInterior,
        )?;

        Ok(Type::Array(Box::new(ArrayType {
            element: Box::new(element),
            length,
        })))
    }
}
