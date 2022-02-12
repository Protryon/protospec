use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}
