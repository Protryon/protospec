use super::*;

#[derive(PartialEq, Clone, Debug)]
pub struct ContainerType {
    pub length: Option<Expression>,
    pub items: IndexMap<String, Arc<Field>>,
    pub is_enum: Cell<bool>,
}

impl ContainerType {
    //todo: optimize this
    pub fn flatten_view<'a>(&'a self) -> impl Iterator<Item = (String, Arc<Field>)> + 'a {
        self.items
            .iter()
            .flat_map(|(name, field)| match &*field.type_.borrow() {
                Type::Container(x) => x.flatten_view().collect::<Vec<_>>(),
                _ => vec![(name.clone(), field.clone())],
            })
    }
}
