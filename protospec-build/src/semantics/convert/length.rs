use super::*;


impl Scope {
    pub fn convert_length(
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
                        PartialType::Array(Some(Box::new(PartialType::Scalar(PartialScalarType::Some(
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