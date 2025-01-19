use std::fmt;

use crate::core::{expression::LinearExpr, objective::ObjectiveSense};

use super::standard_variable::StdVarRef;

#[derive(Debug, Clone)]
pub struct StandardObjective {
    expression: LinearExpr<StdVarRef>,
}

impl StandardObjective {
    pub fn new(expression: LinearExpr<StdVarRef>) -> Self {
        Self { expression }
    }

    pub fn from_sense(sense: &ObjectiveSense, expression: LinearExpr<StdVarRef>) -> Self {
        Self {
            expression: match sense {
                ObjectiveSense::Minimize => -expression,
                ObjectiveSense::Maximize => expression,
            },
        }
    }

    pub fn get_expr(&self) -> &LinearExpr<StdVarRef> {
        &self.expression
    }
}

impl fmt::Display for StandardObjective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Maximize {}", self.expression)
    }
}
