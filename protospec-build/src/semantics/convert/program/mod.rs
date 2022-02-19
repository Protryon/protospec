use super::*;

mod resolve;

mod ffi;

mod import;

mod field;

mod const_;

impl Program {
    pub fn from_ast<'a, T: ImportResolver + 'static>(
        ast: &ast::Program,
        resolver: &'a T,
    ) -> AsgResult<Program> {
        let mut cached_imports: IndexMap<String, Program> = IndexMap::new();

        Program::from_ast_imports(ast, resolver, &mut cached_imports)?;
        Program::from_ast_imported(ast, resolver, &cached_imports)
    }

    fn from_ast_imported<T: ImportResolver + 'static>(
        ast: &ast::Program,
        resolver: &T,
        import_cache: &IndexMap<String, Program>,
    ) -> AsgResult<Program> {
        let program = Arc::new(RefCell::new(Program {
            types: IndexMap::new(),
            consts: IndexMap::new(),
            transforms: IndexMap::new(),
            functions: IndexMap::new(),
        }));

        {
            let mut return_fields = vec![];
            let scope = Arc::new(RefCell::new(Scope {
                parent_scope: None,
                program: program.clone(),
                declared_fields: IndexMap::new(),
                declared_inputs: IndexMap::new(),
            }));

            // import ffis
            for declaration in ast.declarations.iter() {
                match declaration {
                    ast::Declaration::Ffi(ffi) => {
                        Scope::convert_ffi_declaration(ffi, resolver, &*program)?;
                    }
                    _ => (),
                }
            }

            // handle file imports
            for declaration in ast.declarations.iter() {
                match declaration {
                    ast::Declaration::Import(import) => {
                        Scope::convert_import_declaration(import, resolver, &*program, import_cache)?;
                    }
                    _ => (),
                }
            }

            // consts and enums
            for declaration in ast.declarations.iter() {
                match declaration {
                    ast::Declaration::Type(type_) if matches!(type_.value.type_.raw_type, ast::RawType::Enum(_)) => {
                        let field = Scope::convert_type_declaration(type_, &*program)?;
                        let scope = Scope::convert_ast_field_arguments(&scope, &field, Some(&type_.arguments[..]))?;
                        Scope::convert_ast_field(&scope, &type_.value, &field)?;
                    }
                    ast::Declaration::Const(const_) => {
                        Scope::convert_const_declaration(const_, &*program, &scope)?;
                    }
                    _ => (),
                }
            }

            // remaining fields
            for declaration in ast.declarations.iter() {
                match declaration {
                    ast::Declaration::Type(type_) if !matches!(type_.value.type_.raw_type, ast::RawType::Enum(_)) => {
                        let field = Scope::convert_type_declaration(type_, &*program)?;
                        return_fields.push((type_, field));
                    }
                    _ => (),
                }
            }

            // convert arguments
            let mut sub_scopes = vec![];
            for (type_, field) in &return_fields {
                sub_scopes.push(Scope::convert_ast_field_arguments(&scope, &field, Some(&type_.arguments[..]))?);
            }

            // convert rest
            for ((type_, field), sub_scope) in return_fields.into_iter().zip(sub_scopes.iter()) {
                Scope::convert_ast_field(sub_scope, &type_.value, &field)?;
            }
        }

        let program = Arc::try_unwrap(program)
            .ok()
            .expect("leaked program arc")
            .into_inner();

        program.scan_cycles();
        Ok(program)
    }
}
