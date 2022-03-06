use super::*;

#[derive(Debug, Clone)]
pub struct TypeArgument {
    pub name: String,
    pub type_: Type,
    pub default_value: Option<Expression>,
    pub can_resolve_auto: bool,
}

#[derive(Debug)]
pub struct TypeTransform {
    pub transform: Arc<Transform>,
    pub condition: Option<Expression>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub arguments: RefCell<Vec<TypeArgument>>,
    pub span: Span,
    pub type_: RefCell<Type>,
    pub calculated: RefCell<Option<Expression>>,
    pub condition: RefCell<Option<Expression>>,
    pub transforms: RefCell<Vec<TypeTransform>>,
    pub toplevel: bool,
    pub is_maybe_cyclical: Cell<bool>,
    pub is_pad: Cell<bool>,
}

impl Field {
    pub(super) fn get_indirect_contained_fields(&self, target: &mut IndexSet<String>) {
        let type_ = self.type_.borrow();
        type_.get_indirect_contained_fields(target);
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.type_.borrow())?;
        if let Some(condition) = &*self.condition.borrow() {
            write!(f, " {{ {} }}", condition)?;
        }
        for transform in self.transforms.borrow().iter() {
            write!(f, " -> {}", transform.transform.name)?;
            if let Some(condition) = transform.condition.as_ref() {
                write!(f, " {{ {} }}", condition)?;
            }
        }
        Ok(())
    }
}

impl AsgExpression for Field {
    fn get_type(&self) -> Option<Type> {
        Some(self.type_.borrow().clone())
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Field) -> bool {
        self.name == other.name && self.type_.borrow().assignable_from(&other.type_.borrow())
    }
}
