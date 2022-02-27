use crate::ConstDeclaration;

use super::*;

impl Scope {
    pub(super) fn convert_const_declaration(
        const_: &ConstDeclaration,
        program: &RefCell<Program>,
        scope: &Arc<RefCell<Scope>>,
    ) -> AsgResult<()> {
        if let Some(defined) = program.borrow().consts.get(&const_.name.name) {
            return Err(AsgError::ConstRedefinition(
                const_.name.name.clone(),
                const_.span,
                defined.span,
            ));
        }
        let type_ = Scope::convert_ast_type(&scope, &const_.type_.raw_type, TypePurpose::ConstDefinition)?;
        match type_ {
            Type::Container(_) | Type::Enum(_) | Type::Bitfield(_) => {
                return Err(AsgError::ConstTypeDefinition(
                    const_.name.name.clone(),
                    const_.span,
                ));
            }
            _ => (),
        }
        let value = Scope::convert_expr(&scope, &const_.value, type_.clone().into())?;
        program.borrow_mut().consts.insert(
            const_.name.name.clone(),
            Arc::new(Const {
                name: const_.name.name.clone(),
                span: const_.span,
                type_,
                value,
            }),
        );
        Ok(())
    }
}