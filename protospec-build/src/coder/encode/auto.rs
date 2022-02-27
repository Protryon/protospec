use super::*;

impl Context {

    fn resolve_auto(&mut self, field: &Arc<Field>, source: usize) -> Option<usize> {
        let type_ = field.type_.borrow();
        let cast_type = match type_.resolved().as_ref() {
            Type::Scalar(s) => *s,
            Type::Foreign(f) => match f.obj.can_receive_auto() {
                Some(s) => s,
                None => unimplemented!("bad ffi type for auto field"),
            },
            _ => unimplemented!("bad type for auto field"),
        };

        let target = self.alloc_register();
        self.instructions.push(Instruction::GetLen(
            target,
            source,
            Some(cast_type),
        ));
        self.resolved_autos.insert(field.name.clone(), target);
        Some(target)
    }

    pub fn check_auto(&mut self, base: &Expression, source: usize) -> Option<usize> {
        match base {
            Expression::FieldRef(f) if f.is_auto.get() => {
                self.resolve_auto(f, source)
            },
            Expression::FieldRef(_) => None,
            Expression::Binary(_) => None,
            Expression::Member(_) => None,
            Expression::Unary(_) => None,
            Expression::Cast(expr) => self.check_auto(&*expr.inner, source),
            Expression::ArrayIndex(_) => None,
            Expression::EnumAccess(_) => None,
            Expression::Int(_) => None,
            Expression::ConstRef(_) => None,
            Expression::InputRef(_) => None,
            Expression::Str(_) => None,
            Expression::Ternary(_) => None,
            Expression::Bool(_) => None,
            Expression::Call(_) => None,
        }
    }
}