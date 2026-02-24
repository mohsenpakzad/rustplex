use crate::{common::expression::LinearExpr, standard_form::variable::StandardVariableKey};
use std::fmt;

#[derive(Debug, Clone)]
pub struct StandardObjective {
    expression: LinearExpr<StandardVariableKey>,
}

impl StandardObjective {
    pub fn new(expression: LinearExpr<StandardVariableKey>) -> Self {
        Self { expression }
    }

    pub fn expr(&self) -> &LinearExpr<StandardVariableKey> {
        &self.expression
    }
}

impl fmt::Display for StandardObjective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Maximize {}", self.expression)
    }
}
