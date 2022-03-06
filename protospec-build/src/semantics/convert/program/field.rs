use crate::TypeDeclaration;

use super::*;

impl Scope {
    pub(super) fn convert_type_declaration(
        type_: &TypeDeclaration,
        program: &RefCell<Program>,
    ) -> AsgResult<Arc<Field>> {
        if let Some(defined) = program.borrow().types.get(&type_.name.name) {
            return Err(AsgError::TypeRedefinition(
                type_.name.name.clone(),
                type_.span,
                defined.span,
            ));
        }

        let field = Arc::new(Field {
            name: type_.name.name.clone(),
            arguments: RefCell::new(vec![]),
            span: type_.value.span,
            type_: RefCell::new(Type::Bool), // placeholder
            condition: RefCell::new(None),
            calculated: RefCell::new(None),
            transforms: RefCell::new(vec![]),
            toplevel: true,
            is_maybe_cyclical: Cell::new(false),
            is_pad: Cell::new(false),
        });

        program
            .borrow_mut()
            .types
            .insert(type_.name.name.clone(), field.clone());
        Ok(field)
    }
}
