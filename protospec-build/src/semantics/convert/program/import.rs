use crate::ImportDeclaration;

use super::*;

impl Scope {
    pub(super) fn convert_import_declaration<T: ImportResolver + 'static>(
        import: &ImportDeclaration,
        resolver: &T,
        program: &RefCell<Program>,
        import_cache: &IndexMap<String, Program>,
    ) -> AsgResult<()> {
        let content = String::from_utf8_lossy(&import.from.content[..]);
        let normalized = resolver.normalize_import(content.as_ref())?;
        if let Some(cached) = import_cache.get(&normalized) {
            for import_item in import.items.iter() {
                let imported_name = if let Some(alias) = import_item.alias.as_ref()
                {
                    alias.name.clone()
                } else {
                    import_item.name.name.clone()
                };
                if let Some(t) = cached.types.get(&import_item.name.name) {
                    program.borrow_mut().types.insert(imported_name, t.clone());
                } else if let Some(t) = cached.consts.get(&import_item.name.name) {
                    program.borrow_mut().consts.insert(imported_name, t.clone());
                } else if let Some(t) =
                    cached.transforms.get(&import_item.name.name)
                {
                    program
                        .borrow_mut()
                        .transforms
                        .insert(imported_name, t.clone());
                } else {
                    return Err(AsgError::ImportUnresolved(
                        import_item.name.name.clone(),
                        normalized.clone(),
                        import_item.name.span,
                    ));
                }
            }
        } else {
            return Err(AsgError::ImportMissing(normalized, import.span));
        }
        Ok(())
    }
}