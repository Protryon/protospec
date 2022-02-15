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

impl Scope {

    pub fn convert_expr(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::Expression,
        expected_type: PartialType,
    ) -> AsgResult<Expression> {
        use ast::Expression::*;
        Ok(match expr {
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
        })
    }
}