use crate::ast;
use crate::result::*;
use crate::ImportResolver;
use crate::Span;
use crate::{asg::*, ScalarType};
use ast::Node;
use indexmap::IndexMap;
use std::cell::{Cell, RefCell};
use std::fmt;
use std::{sync::Arc, unimplemented};
use thiserror::Error;

pub type AsgResult<T> = StdResult<T, AsgError>;

#[derive(Error)]
pub enum AsgError {
    #[error("unresolved ffi import '{0}' @ {1}")]
    FfiMissing(String, Span),
    #[error("unresolved import '{0}' @ {1}")]
    ImportMissing(String, Span),
    #[error("unresolved import item '{0}' does not exist in module {1} @ {2}")]
    ImportUnresolved(String, String, Span),
    #[error("failed to parse import file '{0}' @ {1}: {2}")]
    ImportParse(String, Span, crate::parser::ParseError),
    #[error("type name already in use: '{0}' @ {1}, originally declared at {2}")]
    TypeRedefinition(String, Span, Span),
    #[error("transform name already in use: '{0}' @ {1}, originally declared at {2}")]
    TransformRedefinition(String, Span, Span),
    #[error("function name already in use: '{0}' @ {1}, originally declared at {2}")]
    FunctionRedefinition(String, Span, Span),
    #[error("const name already in use: '{0}' @ {1}, originally declared at {2}")]
    ConstRedefinition(String, Span, Span),
    #[error("const cannot declare complex type: '{0}' @ {1}")]
    ConstTypeDefinition(String, Span),
    #[error("cast cannot declare complex type @ {0}")]
    CastTypeDefinition(Span),
    #[error("complex types cannot be declared in this context @ {0}")]
    IllegalComplexTypeDefinition(Span),
    #[error("enum variant name already in use: '{0}' @ {1}, originally declared at {2}")]
    EnumVariantRedefinition(String, Span, Span),
    #[error("container field name already in use: '{0}' @ {1}, originally declared at {2}")]
    ContainerFieldRedefinition(String, Span, Span),
    #[error("referenced type '{0}' @ {1} not found")]
    UnresolvedType(String, Span),
    #[error("referenced variable '{0}' @ {1} not found")]
    UnresolvedVar(String, Span),
    #[error("referenced transform '{0}' @ {1} not found")]
    UnresolvedTransform(String, Span),
    #[error("referenced function '{0}' @ {1} not found")]
    UnresolvedFunction(String, Span),
    #[error("referenced transform '{0}' @ {1} cannot encode type {2}")]
    InvalidTransformInput(String, Span, String),
    #[error("referenced transform '{0}' @ {1} cannot cannot have condition because its target encoding type is not assignable to its input encoding type: {2} != {3}")]
    InvalidTransformCondition(String, Span, String, String),
    #[error("unexpected type got {0}, expected {1} @ {2}")]
    UnexpectedType(String, String, Span),
    #[error("illegal cast, cannot cast from {0} to {1} @ {2}")]
    IllegalCast(String, String, Span),
    #[error("reference enum variant for enum {0}, {1}@ {2} is not a valid variant")]
    UnresolvedEnumVariant(String, String, Span),
    #[error("could not infer type @ {0} (try adding more explicit types)")]
    UninferredType(Span),
    #[error("could not parse int {0} @ {1}")]
    InvalidInt(String, Span),
    #[error("invalid number of arguments for ffi, expected {0} to {1} arguments, got {2}")]
    InvalidFFIArgumentCount(usize, usize, usize, Span),
    #[error("invalid number of arguments for type, expected {0} to {1} arguments, got {2}")]
    InvalidTypeArgumentCount(usize, usize, usize, Span),
    #[error("cannot have required arguments after optional arguments for type")]
    InvalidTypeArgumentOrder(Span),
    #[error("illegal repitition of container or enum, outline the container/enum as a top level type declaration")]
    InlineRepetition(Span),
    #[error("invalid or unknown flag")]
    InvalidFlag(Span),
    #[error("unknown")]
    Unknown(#[from] crate::Error),
}

impl fmt::Debug for AsgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
pub enum PartialType {
    Type(Type),
    Scalar(Option<ScalarType>),
    Array(Option<Box<PartialType>>),
    Any,
}

impl PartialType {
    fn assignable_from(&self, other: &Type) -> bool {
        match (self, other.resolved().as_ref()) {
            (t1, Type::Foreign(f2)) => f2.obj.assignable_to_partial(t1),
            (PartialType::Scalar(scalar_type), Type::Enum(e1)) => {
                if let Some(scalar_type) = scalar_type {
                    e1.rep.can_implicit_cast_to(scalar_type)
                } else {
                    true
                }
            }
            (PartialType::Type(x), other) => x.assignable_from(other),
            (PartialType::Scalar(x), Type::Scalar(y)) => {
                x.map(|x| y.can_implicit_cast_to(&x)).unwrap_or(true)
            }
            (PartialType::Array(None), Type::Array(_)) => true,
            (PartialType::Array(Some(element)), Type::Array(array_type)) => {
                element.assignable_from(&array_type.element.type_.borrow())
            }
            (PartialType::Any, _) => true,
            _ => false,
        }
    }
}

impl fmt::Display for PartialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartialType::Type(t) => t.fmt(f),
            PartialType::Scalar(Some(s)) => s.fmt(f),
            PartialType::Scalar(None) => write!(f, "integer"),
            PartialType::Array(None) => write!(f, "array"),
            PartialType::Array(Some(inner)) => write!(f, "{}[]", inner),
            PartialType::Any => write!(f, "any"),
        }
    }
}

impl Into<PartialType> for Type {
    fn into(self) -> PartialType {
        match self {
            Type::Ref(x) => x.target.type_.borrow().clone().into(),
            Type::Scalar(x) => PartialType::Scalar(Some(x)),
            Type::Array(x) => PartialType::Array(Some(Box::new(x.element.type_.borrow().clone().into()))),
            x => PartialType::Type(x),
        }
    }
}

#[derive(Debug)]
struct Scope {
    parent_scope: Option<Arc<RefCell<Scope>>>,
    program: Arc<RefCell<Program>>,
    declared_fields: IndexMap<String, Arc<Field>>,
    declared_inputs: IndexMap<String, Arc<Input>>,
}

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
                                    return Err(AsgError::ImportParse(
                                        content,
                                        import.from.span,
                                        e,
                                    ))
                                }
                            };
                            Program::from_ast_imports(&parsed, resolver, cache)?;
                            let asg = Program::from_ast_imported(&parsed, resolver, cache)?;
                            cache.insert(normalized, asg);
                        } else {
                            return Err(AsgError::ImportMissing(
                                content,
                                import.from.span,
                            ));
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
                                        type_: RefCell::new(Type::Foreign(Arc::new(NamedForeignType {
                                            name: ffi.name.name.clone(),
                                            span: ffi.span,
                                            obj,
                                        }))),
                                        condition: RefCell::new(None),
                                        transforms: RefCell::new(vec![]),
                                        span: ffi.span,
                                        toplevel: true,
                                        is_auto: Cell::new(false),
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
                        let type_ = Scope::convert_ast_type(&scope, &cons.type_.raw_type)?;
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
                Scope::convert_ast_field(
                    &scope,
                    &typ.value,
                    field,
                    Some(&typ.arguments[..]),
                )?;
    
            }    
        }

        Ok(Arc::try_unwrap(program)
            .ok()
            .expect("leaked program arc")
            .into_inner())
    }
}

impl Scope {
    fn resolve_field(self_: &Arc<RefCell<Scope>>, name: &str) -> Option<Arc<Field>> {
        if let Some(field) = self_.borrow().declared_fields.get(name) {
            Some(field.clone())
        } else if let Some(parent) = self_.borrow().parent_scope.as_ref() {
            Scope::resolve_field(parent, name)
        } else {
            None
        }
    }

    fn resolve_input(self_: &Arc<RefCell<Scope>>, name: &str) -> Option<Arc<Input>> {
        if let Some(field) = self_.borrow().declared_inputs.get(name) {
            Some(field.clone())
        } else if let Some(parent) = self_.borrow().parent_scope.as_ref() {
            Scope::resolve_input(parent, name)
        } else {
            None
        }
    }

    fn convert_ffi_arguments(
        self_: &Arc<RefCell<Scope>>,
        name: &str,
        span: Span,
        arguments: &[ast::Expression],
        type_arguments: &[FFIArgument],
    ) -> AsgResult<Vec<Expression>> {
        let min_arg_count = type_arguments
            .iter()
            .filter(|x| !x.optional)
            .count();
        // optionals MUST be at the end
        if min_arg_count < arguments.len()
            && type_arguments[type_arguments.len() - min_arg_count..]
                .iter()
                .any(|x| !x.optional)
        {
            unimplemented!("cannot have ffi transform with required arguments after optional arguments for '{}'", name);
        }
        if arguments.len() < min_arg_count || arguments.len() > type_arguments.len() {
            return Err(AsgError::InvalidFFIArgumentCount(
                min_arg_count,
                type_arguments.len(),
                arguments.len(),
                span,
            ));
        }
        let arguments = arguments
            .iter()
            .zip(type_arguments.iter())
            .map(|(expr, argument)| {
                Scope::convert_expr(self_, expr, argument.type_.clone().map(|x| x.into()).unwrap_or_else(|| PartialType::Any))
            })
            .collect::<AsgResult<Vec<Expression>>>()?;
        Ok(arguments)
    }

    fn convert_ast_field(
        self_: &Arc<RefCell<Scope>>,
        field: &ast::Field,
        into: &Arc<Field>,
        ast_arguments: Option<&[ast::TypeArgument]>,
    ) -> AsgResult<()> {
        let sub_scope = Arc::new(RefCell::new(Scope {
            parent_scope: Some(self_.clone()),
            program: self_.borrow().program.clone(),
            declared_fields: IndexMap::new(),
            declared_inputs: IndexMap::new(),
        }));

        let mut arguments = vec![];
        if let Some(ast_arguments) = ast_arguments {
            for argument in ast_arguments {
                let target_type = Scope::convert_ast_type(
                    &sub_scope,
                    &argument.type_.raw_type,
                )?;
                sub_scope.borrow_mut().declared_inputs.insert(
                    argument.name.name.clone(),
                    Arc::new(Input {
                        name: argument.name.name.clone(),
                        type_: target_type.clone(),
                    }),
                );
                arguments.push(TypeArgument {
                    name: argument.name.name.clone(),
                    type_: target_type.clone(),
                    default_value: argument
                        .default_value
                        .as_ref()
                        .map(|expr| Scope::convert_expr(&sub_scope, expr, target_type.into()))
                        .transpose()?,
                    can_resolve_auto: false,
                });
            }
        }

        let condition = if let Some(condition) = &field.condition {
            Some(Scope::convert_expr(
                &sub_scope,
                &**condition,
                PartialType::Type(Type::Bool),
            )?)
        } else {
            None
        };

        let asg_type =
            Scope::convert_ast_type(&sub_scope, &field.type_.raw_type)?;

        let mut transforms = vec![];
        for ast::Transform {
            name,
            conditional,
            arguments,
            span,
        } in field.transforms.iter()
        {
            let def_transform = if let Some(def_transform) =
                self_.borrow().program.borrow().transforms.get(&name.name)
            {
                def_transform.clone()
            } else {
                return Err(AsgError::UnresolvedTransform(name.name.clone(), name.span));
            };
            let arguments = Self::convert_ffi_arguments(self_, &*def_transform.name, *span, &arguments[..], &def_transform.arguments[..])?;

            transforms.push(TypeTransform {
                transform: def_transform,
                condition: if let Some(conditional) = conditional {
                    Some(Scope::convert_expr(
                        self_,
                        &**conditional,
                        PartialType::Type(Type::Bool),
                    )?)
                } else {
                    None
                },
                arguments,
            })
        }
        let mut is_auto = false;
        for flag in field.flags.iter() {
            match &*flag.name {
                "auto" => {
                    is_auto = true;
                },
                x => return Err(AsgError::InvalidFlag(flag.span)),
            }
        }

        into.type_.replace(asg_type);
        into.condition.replace(condition);
        into.transforms.replace(transforms);
        into.arguments.replace(arguments);
        into.is_auto.replace(is_auto);

        Ok(())
    }

    fn convert_ast_type(
        self_: &Arc<RefCell<Scope>>,
        typ: &ast::RawType,
    ) -> AsgResult<Type> {
        Ok(match typ {
            ast::RawType::Container(value) => {
                let length = value
                    .length
                    .as_ref()
                    .map(|x| {
                        Scope::convert_expr(self_, &**x, PartialType::Scalar(Some(ScalarType::U64)))
                    })
                    .transpose()?;
                let mut items: IndexMap<String, Arc<Field>> = IndexMap::new();
                let sub_scope = Arc::new(RefCell::new(Scope {
                    parent_scope: Some(self_.clone()),
                    program: self_.borrow().program.clone(),
                    declared_fields: IndexMap::new(),
                    declared_inputs: IndexMap::new(),
                }));
                for (name, typ) in value.items.iter() {
                    if let Some(defined) = items.get(&name.name) {
                        return Err(AsgError::ContainerFieldRedefinition(
                            name.name.clone(),
                            name.span,
                            defined.span,
                        ));
                    }
                    let field_out = Arc::new(Field {
                        name: name.name.clone(),
                        type_: RefCell::new(Type::Bool),
                        condition: RefCell::new(None),
                        transforms: RefCell::new(vec![]),
                        span: typ.span,
                        toplevel: false,
                        arguments: RefCell::new(vec![]),
                        is_auto: Cell::new(false),
                    });
            
                    Scope::convert_ast_field(
                        &sub_scope,
                        typ,
                        &field_out,
                        None,
                    )?;

                    sub_scope
                        .borrow_mut()
                        .declared_fields
                        .insert(name.name.clone(), field_out.clone());
                    items.insert(name.name.clone(), field_out);
                }

                Type::Container(Box::new(ContainerType { length, items }))
            }
            ast::RawType::Enum(value) => {
                let mut items: IndexMap<String, Arc<Const>> = IndexMap::new();
                let mut last_defined_item = None::<Arc<Const>>;
                let mut undefined_counter = 0usize;
                for (name, item) in value.items.iter() {
                    if let Some(prior) = items.get(&name.name) {
                        return Err(AsgError::EnumVariantRedefinition(
                            name.name.clone(),
                            name.span,
                            prior.span,
                        ));
                    }
                    //todo: static eval here
                    let cons = Arc::new(Const {
                        name: name.name.clone(),
                        span: value.span,
                        type_: Type::Scalar(value.rep),
                        value: match item {
                            Some(expr) => Scope::convert_expr(
                                self_,
                                &**expr,
                                PartialType::Scalar(Some(value.rep)),
                            )?,
                            None => Expression::Binary(BinaryExpression {
                                op: crate::BinaryOp::Add,
                                left: Box::new(Expression::ConstRef(
                                    last_defined_item.as_ref().unwrap().clone(),
                                )),
                                right: Box::new(Expression::Int(Int {
                                    value: ConstInt::parse(
                                        value.rep,
                                        &*format!("{}", undefined_counter),
                                        name.span,
                                    )?,
                                    type_: value.rep,
                                    span: name.span,
                                })),
                                span: value.span,
                            }),
                        },
                    });
                    if item.is_some() {
                        last_defined_item = Some(cons.clone());
                        undefined_counter = 1;
                    } else {
                        undefined_counter += 1;
                    }
                    items.insert(name.name.clone(), cons);
                }
                Type::Enum(EnumType {
                    rep: value.rep,
                    items,
                })
            }
            ast::RawType::Scalar(value) => Type::Scalar(value.clone()),
            ast::RawType::Array(value) => {
                let length = Scope::convert_length(self_, &value.length)?;
                let element = Scope::convert_ast_type(self_, &value.element.type_.raw_type)?;
                match &element {
                    Type::Container(_) | Type::Enum(_) => {
                        return Err(AsgError::InlineRepetition(value.span));
                    },
                    _ => (),
                }
                let field = Arc::new(Field {
                    name: "$array_field".to_string(),
                    type_: RefCell::new(element),
                    arguments: RefCell::new(vec![]),
                    condition: RefCell::new(None),
                    transforms: RefCell::new(vec![]),
                    span: value.span,
                    toplevel: false,
                    is_auto: Cell::new(false),
                });

                Type::Array(Box::new(ArrayType {
                    element: field,
                    length,
                }))
            }
            ast::RawType::F32 => Type::F32,
            ast::RawType::F64 => Type::F64,
            ast::RawType::Bool => Type::Bool,
            ast::RawType::Ref(call) => {
                if let Some(target) = self_.borrow().program.borrow().types.get(&call.name.name) {
                    let target_args = target.arguments.borrow();
                    let min_arg_count = target_args
                        .iter()
                        .filter(|x| x.default_value.is_some())
                        .count();
                    // optionals MUST be at the end
                    if min_arg_count < call.arguments.len()
                        && target_args[target_args.len() - min_arg_count..]
                            .iter()
                            .any(|x| x.default_value.is_some())
                    {
                        return Err(AsgError::InvalidTypeArgumentOrder(call.span));
                    }
                    if call.arguments.len() < min_arg_count
                        || call.arguments.len() > target_args.len()
                    {
                        return Err(AsgError::InvalidTypeArgumentCount(
                            min_arg_count,
                            target_args.len(),
                            call.arguments.len(),
                            call.span,
                        ));
                    }
                    let arguments = call
                        .arguments
                        .iter()
                        .zip(target_args.iter())
                        .map(|(expr, argument)| {
                            Scope::convert_expr(self_, expr, argument.type_.clone().into())
                        })
                        .collect::<AsgResult<Vec<Expression>>>()?;

                    Type::Ref(TypeCall {
                        target: target.clone(),
                        arguments,
                    })
                } else {
                    return Err(AsgError::UnresolvedType(
                        call.name.name.clone(),
                        call.name.span,
                    ));
                }
            }
        })
    }

    fn convert_expr(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::Expression,
        expected_type: PartialType,
    ) -> AsgResult<Expression> {
        use ast::Expression::*;
        Ok(match expr {
            Binary(expr) => {
                use ast::BinaryOp::*;
                match expr.op {
                    Lt | Gt | Lte | Gte | Eq | Ne | Or | And => {
                        if !expected_type.assignable_from(&Type::Bool) {
                            return Err(AsgError::UnexpectedType(
                                "bool".to_string(),
                                expected_type.to_string(),
                                expr.span,
                            ));
                        }
                    }
                    _ => {
                        // deferred to concrete scalar type
                        // match expected_type {
                        //     PartialType::Scalar(_) => (),
                        //     _ => return Err(AsgError::UnexpectedType("integer".to_string(), expected_type.to_string(), expr.span)),
                        // }
                    }
                }
                let init_expected_type = match expr.op {
                    Lt | Gt | Lte | Gte => PartialType::Scalar(None),
                    Eq | Ne => PartialType::Any,
                    Or | And => PartialType::Type(Type::Bool),
                    _ => expected_type.clone(),
                };
                let mut left = Scope::convert_expr(self_, &expr.left, init_expected_type.clone());
                let right =
                    if let Some(left_type) = left.as_ref().map(|x| x.get_type()).ok().flatten() {
                        Scope::convert_expr(self_, &expr.right, left_type.into())?
                    } else {
                        let right = Scope::convert_expr(self_, &expr.right, init_expected_type)?;
                        if let Some(right_type) = right.get_type() {
                            left = Ok(Scope::convert_expr(self_, &expr.left, right_type.into())?);
                            if left.as_ref().unwrap().get_type().is_none() {
                                return Err(AsgError::UninferredType(*expr.left.span()));
                            }
                        } else {
                            return Err(AsgError::UninferredType(expr.span));
                        }
                        right
                    };
                match expr.op {
                    Lt | Gt | Lte | Gte | Eq | Ne | Or | And => {
                        // nop
                    }
                    _ => {
                        // deferred to concrete scalar type
                        let left_type = left.as_ref().unwrap().get_type().unwrap();
                        if !expected_type.assignable_from(&left_type) {
                            return Err(AsgError::UnexpectedType(
                                left_type.to_string(),
                                expected_type.to_string(),
                                expr.span,
                            ));
                        }
                    }
                }
                Expression::Binary(BinaryExpression {
                    op: expr.op.clone(),
                    left: Box::new(left.unwrap()),
                    right: Box::new(right),
                    span: expr.span,
                })
            }
            Unary(expr) => {
                let inner = Box::new(Scope::convert_expr(
                    self_,
                    &expr.inner,
                    expected_type.clone(),
                )?);
                match expr.op {
                    ast::UnaryOp::Not => {
                        if !expected_type.assignable_from(&Type::Bool) {
                            return Err(AsgError::UnexpectedType(
                                "bool".to_string(),
                                expected_type.to_string(),
                                expr.span,
                            ));
                        }
                    }
                    ast::UnaryOp::Negate | ast::UnaryOp::BitNot => {
                        if let Some(inner_type) = inner.get_type() {
                            if !PartialType::Scalar(None).assignable_from(&inner_type) {
                                return Err(AsgError::UnexpectedType(
                                    inner_type.to_string(),
                                    "integer".to_string(),
                                    expr.span,
                                ));
                            }
                            if !expected_type.assignable_from(&inner_type) {
                                return Err(AsgError::UnexpectedType(
                                    inner_type.to_string(),
                                    expected_type.to_string(),
                                    expr.span,
                                ));
                            }
                            if expr.op == ast::UnaryOp::Negate {
                                match inner_type {
                                    Type::Scalar(s) if !s.is_signed() => {
                                        return Err(AsgError::UnexpectedType(
                                            inner_type.to_string(),
                                            "signed integer".to_string(),
                                            expr.span,
                                        ));
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                }
                Expression::Unary(UnaryExpression {
                    op: expr.op.clone(),
                    inner: Box::new(Scope::convert_expr(self_, &expr.inner, expected_type)?),
                    span: expr.span,
                })
            }
            Cast(expr) => {
                match &expr.type_.raw_type {
                    ast::RawType::Container(_) | ast::RawType::Enum(_) => {
                        return Err(AsgError::CastTypeDefinition(expr.span));
                    }
                    _ => (),
                }
                let target = Scope::convert_ast_type(self_, &expr.type_.raw_type)?;
                if !expected_type.assignable_from(&target) {
                    return Err(AsgError::UnexpectedType(
                        target.to_string(),
                        expected_type.to_string(),
                        expr.span,
                    ));
                }

                let inner = Box::new(Scope::convert_expr(self_, &expr.inner, PartialType::Any)?);
                if let Some(inner_type) = inner.get_type() {
                    if !inner_type.can_cast_to(&target) {
                        return Err(AsgError::IllegalCast(
                            inner_type.to_string(),
                            target.to_string(),
                            expr.span,
                        ));
                    }
                } else {
                    return Err(AsgError::UninferredType(*expr.inner.span()));
                }

                Expression::Cast(CastExpression {
                    type_: target,
                    inner,
                    span: expr.span,
                })
            }
            ArrayIndex(expr) => Expression::ArrayIndex(ArrayIndexExpression {
                array: Box::new(Scope::convert_expr(
                    self_,
                    &expr.array,
                    PartialType::Array(Some(Box::new(expected_type))),
                )?),
                index: Box::new(Scope::convert_expr(
                    self_,
                    &expr.index,
                    Type::Scalar(ScalarType::U64).into(),
                )?),
                span: expr.span,
            }),
            EnumAccess(expr) => {
                let field = match self_.borrow().program.borrow().types.get(&expr.name.name) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(AsgError::UnresolvedType(
                            expr.name.name.clone(),
                            expr.name.span,
                        ))
                    }
                };
                let variant = match &*field.type_.borrow() {
                    Type::Enum(e) => e
                        .items
                        .get(&expr.variant.name)
                        .ok_or(AsgError::UnresolvedEnumVariant(
                            field.name.clone(),
                            expr.variant.name.clone(),
                            expr.variant.span,
                        ))?
                        .clone(),
                    _ => {
                        return Err(AsgError::UnexpectedType(
                            field.type_.borrow().to_string(),
                            "enum".to_string(),
                            expr.name.span,
                        ));
                    }
                };
                Expression::EnumAccess(EnumAccessExpression {
                    enum_field: field,
                    variant,
                    span: expr.span,
                })
            }
            Int(expr) => {
                match (&expected_type, &expr.type_) {
                    (x, Some(y)) if x.assignable_from(&Type::Scalar(*y)) => (),
                    (PartialType::Scalar(Some(_)), None) => (),
                    (PartialType::Scalar(None), Some(_)) => (),
                    (PartialType::Any, Some(_)) => (),
                    (x, Some(y)) => {
                        return Err(AsgError::UnexpectedType(
                            y.to_string(),
                            x.to_string(),
                            expr.span,
                        ));
                    }
                    (x, _) => {
                        return Err(AsgError::UnexpectedType(
                            "integer".to_string(),
                            x.to_string(),
                            expr.span,
                        ));
                    }
                }
                let type_ = match (&expected_type, &expr.type_) {
                    (_, Some(s)) => *s,
                    (PartialType::Scalar(Some(s)), _) => *s,
                    _ => unimplemented!(),
                };
                Expression::Int(crate::asg::Int {
                    value: ConstInt::parse(type_, &expr.value, expr.span)?,
                    type_,
                    span: expr.span,
                })
            }
            Bool(expr) => {
                match &expected_type {
                    PartialType::Type(Type::Bool) => (),
                    x => {
                        return Err(AsgError::UnexpectedType(
                            "bool".to_string(),
                            x.to_string(),
                            expr.span,
                        ));
                    }
                }
                Expression::Bool(expr.value)
            }
            Ref(expr) => {
                if let Some(field) = Scope::resolve_field(self_, &expr.name) {
                    Expression::FieldRef(field)
                } else if let Some(input) = Scope::resolve_input(self_, &expr.name) {
                    Expression::InputRef(input)
                } else if let Some(cons) = self_.borrow().program.borrow().consts.get(&expr.name) {
                    Expression::ConstRef(cons.clone())
                } else {
                    return Err(AsgError::UnresolvedVar(expr.name.clone(), expr.span));
                }
            }
            Str(expr) => {
                let out = Expression::Str(expr.clone());
                let out_type = out.get_type().expect("untyped string");
                if !expected_type.assignable_from(&out_type) {
                    return Err(AsgError::UnexpectedType(
                        out_type.to_string(),
                        expected_type.to_string(),
                        expr.span,
                    ));
                }
                out
            }
            Ternary(expr) => {
                let condition = Scope::convert_expr(self_, &expr.condition, Type::Bool.into())?;
                let if_true = Scope::convert_expr(self_, &expr.if_true, expected_type.clone())?;
                let right_type = match expected_type {
                    PartialType::Any => if_true
                        .get_type()
                        .map(|x| x.into())
                        .ok_or(AsgError::UninferredType(*expr.if_true.span()))?,
                    x => x,
                };
                let if_false = Scope::convert_expr(self_, &expr.if_false, right_type)?;
                Expression::Ternary(TernaryExpression {
                    condition: Box::new(condition),
                    if_true: Box::new(if_true),
                    if_false: Box::new(if_false),
                    span: expr.span,
                })
            },
            Call(call) => {
                let scope = self_.borrow();

                let function = scope.program.borrow().functions.get(&*call.function.name)
                    .ok_or_else(|| AsgError::UnresolvedFunction(call.function.name.clone(), call.function.span))?
                    .clone();
                
                let arguments = Self::convert_ffi_arguments(self_, &*function.name, call.span, &call.arguments[..], &function.arguments[..])?;
                
                Expression::Call(CallExpression {
                    function,
                    arguments,
                    span: call.span,
                })
            },
        })
    }

    fn convert_length(
        self_: &Arc<RefCell<Scope>>,
        typ: &ast::LengthConstraint,
    ) -> AsgResult<LengthConstraint> {
        Ok(LengthConstraint {
            expandable: typ.expandable,
            value: if let Some(inner) = &typ.inner {
                Some(Scope::convert_expr(
                    self_,
                    inner,
                    if typ.expandable {
                        PartialType::Array(Some(Box::new(PartialType::Scalar(Some(
                            ScalarType::U8,
                        )))))
                    } else {
                        Type::Scalar(ScalarType::U64).into()
                    },
                )?)
            } else {
                None
            },
        })
    }
}
