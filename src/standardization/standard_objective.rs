use std::fmt;

use crate::core::{expression::LinearExpr, objective::ObjectiveSense};

use super::standard_variable::StdVar;

#[derive(Debug, Clone)]
pub struct StandardObjective {
    expression: LinearExpr<StdVar>,
}

impl StandardObjective {
    pub fn new(expression: LinearExpr<StdVar>) -> Self {
        Self { expression }
    }

    pub fn from_sense(sense: &ObjectiveSense, expression: LinearExpr<StdVar>) -> Self {
        Self {
            expression: match sense {
                ObjectiveSense::Minimize => -expression,
                ObjectiveSense::Maximize => expression,
            },
        }
    }

    pub fn expr(&self) -> &LinearExpr<StdVar> {
        &self.expression
    }
}

impl fmt::Display for StandardObjective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Maximize {}", self.expression)
    }
}
