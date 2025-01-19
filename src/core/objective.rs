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

    pub fn get_sense(&self) -> &ObjectiveSense {
        &self.sense
    }

    pub fn get_expr(&self) -> &LinearExpr<VarRef> {
        &self.expression
    }
}
