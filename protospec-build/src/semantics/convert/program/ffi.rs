use crate::{FfiDeclaration, ForeignFunctionObj};

use super::*;

impl Scope {
    pub(super) fn add_prelude_ffi<T: ImportResolver + 'static>(
        resolver: &T,
        program: &RefCell<Program>,
    ) -> AsgResult<()> {
        for (name, function) in resolver.prelude_ffi_functions()? {
            Self::import_function(&mut program.borrow_mut(), &*name, function);
        }
        Ok(())
    }

    pub(super) fn convert_ffi_declaration<T: ImportResolver + 'static>(
        ffi: &FfiDeclaration,
        resolver: &T,
        program: &RefCell<Program>,
    ) -> AsgResult<()> {
        match ffi.ffi_type {
            ast::FfiType::Type => {
                if let Some(obj) = resolver.resolve_ffi_type(&ffi.name.name)? {
                    if let Some(defined) = program.borrow().types.get(&ffi.name.name) {
                        return Err(AsgError::TypeRedefinition(
                            ffi.name.name.clone(),
                            ffi.span,
                            defined.span,
                        ));
                    }
                    let field = Arc::new(Field {
                        name: ffi.name.name.clone(),
                        arguments: RefCell::new(obj.arguments()),
                        type_: RefCell::new(Type::Foreign(Arc::new(ForeignType {
                            name: ffi.name.name.clone(),
                            span: ffi.span,
                            obj,
                        }))),
                        calculated: RefCell::new(None),
                        condition: RefCell::new(None),
                        transforms: RefCell::new(vec![]),
                        span: ffi.span,
                        toplevel: true,
                        is_maybe_cyclical: Cell::new(false),
                        is_pad: Cell::new(false),
                    });

                    program
                        .borrow_mut()
                        .types
                        .insert(ffi.name.name.clone(), field.clone());
                } else {
                    return Err(AsgError::FfiMissing(ffi.name.name.clone(), ffi.span));
                }
            }
            ast::FfiType::Transform => {
                if let Some(obj) = resolver.resolve_ffi_transform(&ffi.name.name)? {
                    if let Some(defined) = program.borrow().transforms.get(&ffi.name.name) {
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
                    return Err(AsgError::FfiMissing(ffi.name.name.clone(), ffi.span));
                }
            }
            ast::FfiType::Function => {
                if let Some(obj) = resolver.resolve_ffi_function(&ffi.name.name)? {
                    if let Some(defined) = program.borrow().functions.get(&ffi.name.name) {
                        return Err(AsgError::FunctionRedefinition(
                            ffi.name.name.clone(),
                            ffi.span,
                            defined.span,
                        ));
                    }
                    Self::import_function(&mut program.borrow_mut(), &ffi.name.name, obj);
                } else {
                    return Err(AsgError::FfiMissing(ffi.name.name.clone(), ffi.span));
                }
            }
        }
        Ok(())
    }

    fn import_function(program: &mut Program, name: &str, function: ForeignFunctionObj) {
        program.functions.insert(
            name.to_string(),
            Arc::new(Function {
                name: name.to_string(),
                span: Span::default(),
                arguments: function.arguments(),
                inner: function,
            }),
        );
    }
}
