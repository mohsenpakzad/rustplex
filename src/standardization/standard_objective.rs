use std::fmt;

use crate::core::{expression::LinearExpr, objective::ObjectiveSense};

use super::standard_variable::StandardVariableKey;

#[derive(Debug, Clone)]
pub struct StandardObjective {
    expression: LinearExpr<StandardVariableKey>,
}

impl StandardObjective {
    pub fn new(expression: LinearExpr<StandardVariableKey>) -> Self {
        Self { expression }
    }

    pub fn from_sense(sense: &ObjectiveSense, expression: LinearExpr<StandardVariableKey>) -> Self {
        Self {
            expression: match sense {
                ObjectiveSense::Minimize => -expression,
                ObjectiveSense::Maximize => expression,
            },
        }
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
