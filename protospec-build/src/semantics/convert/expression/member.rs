use super::*;

impl Scope {
    pub(super) fn convert_member_expression(
        self_: &Arc<RefCell<Scope>>,
        expr: &ast::MemberExpression,
        _expected_type: PartialType,
    ) -> AsgResult<MemberExpression> {
        let target = Scope::convert_expr(self_, &expr.target, PartialType::Any)?;
        let type_ = target
            .get_type()
            .ok_or_else(|| AsgError::UninferredType(*expr.target.span()))?;
        let member =
            match &*type_.resolved() {
                Type::Bitfield(bitfield) => bitfield
                    .items
                    .get(&expr.member.name)
                    .cloned()
                    .ok_or_else(|| {
                        AsgError::BitfieldMemberUndefined(
                            expr.member.name.clone(),
                            expr.member.span,
                        )
                    })?,
                t => {
                    return Err(AsgError::UnexpectedType(
                        t.to_string(),
                        "bitfield".to_string(),
                        *expr.target.span(),
                    ))?
                }
            };

        Ok(MemberExpression {
            target: Box::new(target),
            member,
            span: expr.span,
        })
    }
}
