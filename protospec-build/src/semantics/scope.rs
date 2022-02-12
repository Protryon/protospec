use super::*;

#[derive(Debug)]
pub struct Scope {
    pub parent_scope: Option<Arc<RefCell<Scope>>>,
    pub program: Arc<RefCell<Program>>,
    pub declared_fields: IndexMap<String, Arc<Field>>,
    pub declared_inputs: IndexMap<String, Arc<Input>>,
}
