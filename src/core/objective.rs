use crate::core::expression::LinearExpr;

#[derive(Debug, Clone)]
pub struct Objective {
    pub expression: LinearExpr,
    pub sense: ObjectiveSense,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectiveSense {
    Minimize,
    Maximize,
}

impl Objective {
    pub fn new(expression: LinearExpr, sense: ObjectiveSense) -> Self {
        Self { expression, sense }
    }

    pub fn minimize(expression: LinearExpr) -> Self {
        Self::new(expression, ObjectiveSense::Minimize)
    }

    pub fn maximize(expression: LinearExpr) -> Self {
        Self::new(expression, ObjectiveSense::Maximize)
    }
}
