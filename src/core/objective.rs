use crate::core::expression::LinearExpr;

use super::variable::VarRef;

#[derive(Debug, Clone)]
pub struct Objective {
    sense: ObjectiveSense,
    expression: LinearExpr<VarRef>,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectiveSense {
    Minimize,
    Maximize,
}

impl Objective {
    pub fn new(sense: ObjectiveSense, expression: LinearExpr<VarRef>) -> Self {
        Self { sense, expression }
    }

    pub fn get_sense(&self) -> &ObjectiveSense {
        &self.sense
    }

    pub fn get_expr(&self) -> &LinearExpr<VarRef> {
        &self.expression
    }
}
