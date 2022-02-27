use super::*;

impl Scope {
    pub(super) fn convert_array_type(
        self_: &Arc<RefCell<Scope>>,
        type_: &ast::Array,
    ) -> AsgResult<Type> {
        let length = Scope::convert_length(self_, &type_.length)?;
        let element = Scope::convert_ast_type(self_, &type_.element.type_.raw_type, TypePurpose::ArrayInterior)?;
        let field = Arc::new(Field {
            name: "$array_field".to_string(),
            type_: RefCell::new(element),
            arguments: RefCell::new(vec![]),
            condition: RefCell::new(None),
            transforms: RefCell::new(vec![]),
            span: type_.span,
            toplevel: false,
            is_auto: Cell::new(false),
            is_maybe_cyclical: Cell::new(false),
            is_pad: Cell::new(false),
        });

        Ok(Type::Array(Box::new(ArrayType {
            element: field,
            length,
        })))
    }
}
