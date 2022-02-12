use super::*;

#[derive(Debug)]
pub struct Program {
    pub types: IndexMap<String, Arc<Field>>,
    pub consts: IndexMap<String, Arc<Const>>,
    pub transforms: IndexMap<String, Arc<Transform>>,
    pub functions: IndexMap<String, Arc<Function>>,
}

impl Program {
    pub fn scan_cycles(&self) {
        for (_, field) in &self.types {
            let mut interior_fields = IndexSet::new();
            field.get_indirect_contained_fields(&mut interior_fields);
            if interior_fields.contains(&field.name) {
                field.is_maybe_cyclical.set(true);
            }
        }
    }
}
