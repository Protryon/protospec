use super::*;

mod binary;

mod unary;

mod cast;

mod array_index;

mod enum_access;

mod int;

mod boolean;

mod var_ref;

mod string;

mod ternary;

mod call;

mod member;

impl Scope {

    pub fn convert_expr(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::Expression,
        expected_type: PartialType,
    ) -> AsgResult<Expression> {
        use ast::Expression::*;
        let coerce_to = expected_type.clone();
        let expression = match expr {
            Binary(expr) => {
                Expression::Binary(Self::convert_binary_expression(self_, expr, expected_type)?)
            }
            Unary(expr) => {
                Expression::Unary(Self::convert_unary_expression(self_, expr, expected_type)?)
            }
            Cast(expr) => {
                Expression::Cast(Self::convert_cast_expression(self_, expr, expected_type)?)
            }
            ArrayIndex(expr) => {
                Expression::ArrayIndex(Self::convert_array_index_expression(self_, expr, expected_type)?)
            }
            EnumAccess(expr) => {
                Expression::EnumAccess(Self::convert_enum_access_expression(self_, expr, expected_type)?)
            }
            Int(expr) => {
                Expression::Int(Self::convert_int_expression(self_, expr, expected_type)?)
            }
            Bool(expr) => {
                Expression::Bool(Self::convert_bool_expression(self_, expr, expected_type)?)
            }
            Ref(expr) => {
                Self::convert_ref_expression(self_, expr, expected_type)?
            }
            Str(expr) => {
                Self::convert_str_expression(self_, expr, expected_type)?
            }
            Ternary(expr) => {
                Expression::Ternary(Self::convert_ternary_expression(self_, expr, expected_type)?)
            }
            Call(expr) => {
                Expression::Call(Self::convert_call_expression(self_, expr, expected_type)?)
            }
            Member(expr) => {
                Expression::Member(Self::convert_member_expression(self_, expr, expected_type)?)
            }
        };
        
        Ok(expression.assign_to_or_coerce_to(coerce_to, *expr.span())?)
    }
}


impl Expression {
    pub(crate) fn assign_to_or_coerce_to(mut self, expected_type: PartialType, span: Span) -> AsgResult<Self> {
        let type_ = self.get_type().expect("coercion missing type");
        if !expected_type.assignable_from(&type_) {
            if expected_type.coercable_from(&type_) {
                self = Expression::Cast(CastExpression {
                    inner: Box::new(self),
                    type_: expected_type.into_type()
                        .ok_or_else(|| AsgError::UninferredType(span))?,
                    span,
                });
                Ok(self)
            } else {
                Err(AsgError::UnexpectedType(type_.to_string(), expected_type.to_string(), span))
            }
        } else {
            Ok(self)
        }
    }
}