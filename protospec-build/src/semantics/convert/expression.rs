use super::*;

impl Scope {

    pub fn convert_expr(
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
                let target = Scope::convert_ast_type(self_, &expr.type_.raw_type, false)?;
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
            }
            Call(call) => {
                let scope = self_.borrow();

                let function = scope
                    .program
                    .borrow()
                    .functions
                    .get(&*call.function.name)
                    .ok_or_else(|| {
                        AsgError::UnresolvedFunction(call.function.name.clone(), call.function.span)
                    })?
                    .clone();

                let arguments = Self::convert_ffi_arguments(
                    self_,
                    &*function.name,
                    call.span,
                    &call.arguments[..],
                    &function.arguments[..],
                )?;

                Expression::Call(CallExpression {
                    function,
                    arguments,
                    span: call.span,
                })
            }
        })
    }
}