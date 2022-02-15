use super::*;

impl Program {
    pub(super) fn from_ast_imports<T: ImportResolver + 'static>(
        ast: &ast::Program,
        resolver: &T,
        cache: &mut IndexMap<String, Program>,
    ) -> AsgResult<()> {
        for declaration in ast.declarations.iter() {
            match declaration {
                ast::Declaration::Import(import) => {
                    let content = String::from_utf8_lossy(&import.from.content[..]).into_owned();
                    let normalized = resolver.normalize_import(&content[..])?;
                    if let Some(_cached) = cache.get(&normalized) {
                    } else {
                        let loaded = resolver.resolve_import(&normalized)?;
                        if let Some(loaded) = loaded {
                            let parsed = match crate::parse(&loaded) {
                                Ok(x) => x,
                                Err(e) => {
                                    return Err(AsgError::ImportParse(content, import.from.span, e))
                                }
                            };
                            Program::from_ast_imports(&parsed, resolver, cache)?;
                            let asg = Program::from_ast_imported(&parsed, resolver, cache)?;
                            cache.insert(normalized, asg);
                        } else {
                            return Err(AsgError::ImportMissing(content, import.from.span));
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }
}