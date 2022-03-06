use super::*;

impl Scope {
    pub(super) fn convert_enum_type(
        self_: &Arc<RefCell<Scope>>,
        type_: &ast::Enum,
        purpose: TypePurpose,
    ) -> AsgResult<Type> {
        let mut items: IndexMap<String, EnumValue> = IndexMap::new();
        let mut last_defined_item = None::<Arc<Const>>;
        let mut undefined_counter = 0usize;
        let mut has_default = false;
        for (name, item) in type_.items.iter() {
            if let Some(_) = items.get(&name.name) {
                return Err(AsgError::EnumVariantRedefinition(
                    name.name.clone(),
                    name.span,
                ));
            }
            if has_default {
                return Err(AsgError::EnumDefaultRedefinition(
                    name.name.clone(),
                    name.span,
                ));
            }
            //todo: static eval here
            let value = match item {
                ast::EnumValue::Default => {
                    has_default = true;
                    EnumValue::Default
                }
                item => {
                    let value = match item {
                        ast::EnumValue::Default => unreachable!(),
                        ast::EnumValue::Expression(expr) => Scope::convert_expr(
                            self_,
                            &**expr,
                            PartialType::Scalar(PartialScalarType::Some(type_.rep.scalar)),
                        )?,
                        ast::EnumValue::None => Expression::Binary(BinaryExpression {
                            op: crate::BinaryOp::Add,
                            left: Box::new(Expression::ConstRef(
                                last_defined_item.as_ref().unwrap().clone(),
                            )),
                            right: Box::new(Expression::Int(Int {
                                value: ConstInt::parse(
                                    type_.rep.scalar,
                                    &*format!("{}", undefined_counter),
                                    name.span,
                                )?,
                                type_: type_.rep.scalar,
                                span: name.span,
                            })),
                            span: type_.span,
                        }),
                    };
                    EnumValue::Value(Arc::new(Const {
                        name: name.name.clone(),
                        span: type_.span,
                        type_: Type::Scalar(type_.rep),
                        value,
                    }))
                }
            };
            if matches!(item, ast::EnumValue::Expression(_)) {
                last_defined_item = Some(match &value {
                    EnumValue::Value(c) => c.clone(),
                    _ => panic!(),
                });
                undefined_counter = 1;
            } else {
                undefined_counter += 1;
            }
            items.insert(name.name.clone(), value);
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
