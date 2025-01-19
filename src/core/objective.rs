use crate::core::expression::LinearExpr;

use super::variable::VarRef;

#[derive(Debug, Clone)]
pub struct Objective {
    pub expression: LinearExpr<VarRef>,
    pub sense: ObjectiveSense,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectiveSense {
    Minimize,
    Maximize,
}

impl Objective {
    pub fn new(expression: LinearExpr<VarRef>, sense: ObjectiveSense) -> Self {
        Self { expression, sense }
    }

    pub fn minimize(expression: LinearExpr<VarRef>) -> Self {
        Self::new(expression, ObjectiveSense::Minimize)
    }

    pub fn maximize(expression: LinearExpr<VarRef>) -> Self {
        Self::new(expression, ObjectiveSense::Maximize)
    }
}
