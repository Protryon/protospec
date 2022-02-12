use super::*;

impl Program {
    fn from_ast_imports<T: ImportResolver + 'static>(
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

            for declaration in ast.declarations.iter() {
                match declaration {
                    ast::Declaration::Import(import) => {
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
                            panic!("unresolved import: {}", normalized);
                        }
                    }
                    ast::Declaration::Ffi(ffi) => {
                        match ffi.ffi_type {
                            ast::FfiType::Type => {
                                if let Some(obj) = resolver.resolve_ffi_type(&ffi.name.name)? {
                                    if let Some(defined) =
                                        program.borrow().types.get(&ffi.name.name)
                                    {
                                        return Err(AsgError::TypeRedefinition(
                                            ffi.name.name.clone(),
                                            ffi.span,
                                            defined.span,
                                        ));
                                    }
                                    let field = Arc::new(Field {
                                        name: ffi.name.name.clone(),
                                        arguments: RefCell::new(obj.arguments()),
                                        type_: RefCell::new(Type::Foreign(Arc::new(
                                            ForeignType {
                                                name: ffi.name.name.clone(),
                                                span: ffi.span,
                                                obj,
                                            },
                                        ))),
                                        condition: RefCell::new(None),
                                        transforms: RefCell::new(vec![]),
                                        span: ffi.span,
                                        toplevel: true,
                                        is_auto: Cell::new(false),
                                        is_maybe_cyclical: Cell::new(false),
                                    });

                                    program
                                        .borrow_mut()
                                        .types
                                        .insert(ffi.name.name.clone(), field.clone());
                                    // scope.borrow_mut().declared_fields.insert(ffi.name.name.clone(), field);
                                } else {
                                    return Err(AsgError::FfiMissing(
                                        ffi.name.name.clone(),
                                        ffi.span,
                                    ));
                                }
                            }
                            ast::FfiType::Transform => {
                                if let Some(obj) = resolver.resolve_ffi_transform(&ffi.name.name)? {
                                    if let Some(defined) =
                                        program.borrow().transforms.get(&ffi.name.name)
                                    {
                                        return Err(AsgError::TransformRedefinition(
                                            ffi.name.name.clone(),
                                            ffi.span,
                                            defined.span,
                                        ));
                                    }
                                    program.borrow_mut().transforms.insert(
                                        ffi.name.name.clone(),
                                        Arc::new(Transform {
                                            name: ffi.name.name.clone(),
                                            span: ffi.span.clone(),
                                            arguments: obj.arguments(),
                                            inner: obj,
                                        }),
                                    );
                                } else {
                                    return Err(AsgError::FfiMissing(
                                        ffi.name.name.clone(),
                                        ffi.span,
                                    ));
                                }
                            }
                            ast::FfiType::Function => {
                                if let Some(obj) = resolver.resolve_ffi_function(&ffi.name.name)? {
                                    if let Some(defined) =
                                        program.borrow().functions.get(&ffi.name.name)
                                    {
                                        return Err(AsgError::FunctionRedefinition(
                                            ffi.name.name.clone(),
                                            ffi.span,
                                            defined.span,
                                        ));
                                    }
                                    program.borrow_mut().functions.insert(
                                        ffi.name.name.clone(),
                                        Arc::new(Function {
                                            name: ffi.name.name.clone(),
                                            span: ffi.span.clone(),
                                            arguments: obj.arguments(),
                                            inner: obj,
                                        }),
                                    );
                                } else {
                                    return Err(AsgError::FfiMissing(
                                        ffi.name.name.clone(),
                                        ffi.span,
                                    ));
                                }
                            }
                        }
                    }
                    ast::Declaration::Const(cons) => {
                        if let Some(defined) = program.borrow().consts.get(&cons.name.name) {
                            return Err(AsgError::ConstRedefinition(
                                cons.name.name.clone(),
                                cons.span,
                                defined.span,
                            ));
                        }
                        let type_ = Scope::convert_ast_type(&scope, &cons.type_.raw_type, true)?;
                        match type_ {
                            Type::Container(_) | Type::Enum(_) => {
                                return Err(AsgError::ConstTypeDefinition(
                                    cons.name.name.clone(),
                                    cons.span,
                                ));
                            }
                            _ => (),
                        }
                        let value = Scope::convert_expr(&scope, &cons.value, type_.clone().into())?;
                        program.borrow_mut().consts.insert(
                            cons.name.name.clone(),
                            Arc::new(Const {
                                name: cons.name.name.clone(),
                                span: cons.span,
                                type_,
                                value,
                            }),
                        );
                    }
                    ast::Declaration::Type(typ) => {
                        if let Some(defined) = program.borrow().types.get(&typ.name.name) {
                            return Err(AsgError::TypeRedefinition(
                                typ.name.name.clone(),
                                typ.span,
                                defined.span,
                            ));
                        }

                        let field = Arc::new(Field {
                            name: typ.name.name.clone(),
                            arguments: RefCell::new(vec![]),
                            span: typ.value.span,
                            type_: RefCell::new(Type::Bool), // placeholder
                            condition: RefCell::new(None),
                            transforms: RefCell::new(vec![]),
                            toplevel: true,
                            is_auto: Cell::new(false),
                            is_maybe_cyclical: Cell::new(false),
                        });
                        return_fields.push(typ);

                        program
                            .borrow_mut()
                            .types
                            .insert(typ.name.name.clone(), field.clone());
                    }
                }
            }
            for typ in return_fields {
                let program = program.borrow();
                let field = program.types.get(&typ.name.name).unwrap();
                Scope::convert_ast_field(&scope, &typ.value, field, Some(&typ.arguments[..]))?;
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
