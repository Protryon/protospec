use super::*;

impl Scope {
    pub(super) fn convert_array_type(
        self_: &Arc<RefCell<Scope>>,
        type_: &ast::Array,
    ) -> AsgResult<Type> {
        let length = Scope::convert_length(self_, &type_.length)?;
        let element = Scope::convert_ast_type(self_, &type_.element.type_.raw_type, false)?;
        match &element {
            Type::Container(_) | Type::Enum(_) => {
                return Err(AsgError::InlineRepetition(type_.span));
            }
            _ => (),
        }
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
        });

        Ok(Type::Array(Box::new(ArrayType {
            element: field,
            length,
        })))
    }
}
