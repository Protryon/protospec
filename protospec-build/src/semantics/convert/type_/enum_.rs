use super::*;

impl Scope {
    pub(super) fn convert_enum_type(
        self_: &Arc<RefCell<Scope>>,
        type_: &ast::Enum,
        purpose: TypePurpose,
    ) -> AsgResult<Type> {
        let mut items: IndexMap<String, Arc<Const>> = IndexMap::new();
        let mut last_defined_item = None::<Arc<Const>>;
        let mut undefined_counter = 0usize;
        for (name, item) in type_.items.iter() {
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
                span: type_.span,
                type_: Type::Scalar(type_.rep),
                value: match item {
                    Some(expr) => Scope::convert_expr(
                        self_,
                        &**expr,
                        PartialType::Scalar(PartialScalarType::Some(type_.rep)),
                    )?,
                    None => Expression::Binary(BinaryExpression {
                        op: crate::BinaryOp::Add,
                        left: Box::new(Expression::ConstRef(
                            last_defined_item.as_ref().unwrap().clone(),
                        )),
                        right: Box::new(Expression::Int(Int {
                            value: ConstInt::parse(
                                type_.rep,
                                &*format!("{}", undefined_counter),
                                name.span,
                            )?,
                            type_: type_.rep,
                            span: name.span,
                        })),
                        span: type_.span,
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
        let name = match purpose {
            TypePurpose::TypeDefinition(name) => name,
            _ => unreachable!("cannot have a non-toplevel enum"),
        };
        Ok(Type::Enum(EnumType {
            name,
            rep: type_.rep,
            items,
        }))
    }
}
