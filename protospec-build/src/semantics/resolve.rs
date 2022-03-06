use super::*;

impl Scope {
    pub fn resolve_field(self_: &Arc<RefCell<Scope>>, name: &str) -> Option<Arc<Field>> {
        if let Some(field) = self_.borrow().declared_fields.get(name) {
            Some(field.clone())
        } else if let Some(parent) = self_.borrow().parent_scope.as_ref() {
            Scope::resolve_field(parent, name)
        } else {
            None
        }
    }

    pub fn resolve_input(self_: &Arc<RefCell<Scope>>, name: &str) -> Option<Arc<Input>> {
        if let Some(field) = self_.borrow().declared_inputs.get(name) {
            Some(field.clone())
        } else if let Some(parent) = self_.borrow().parent_scope.as_ref() {
            Scope::resolve_input(parent, name)
        } else {
            None
        }
    }
}
