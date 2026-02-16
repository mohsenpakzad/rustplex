use std::fmt;

use crate::core::expression::LinearExpr;

use super::variable::VariableKey;

#[derive(Debug, Clone)]
pub struct Objective {
    sense: ObjectiveSense,
    expression: LinearExpr<VariableKey>,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectiveSense {
    Minimize,
    Maximize,
}

impl Objective {
    pub fn new(sense: ObjectiveSense, expression: LinearExpr<VariableKey>) -> Self {
        Self { sense, expression }
    }

    pub fn sense(&self) -> &ObjectiveSense {
        &self.sense
    }

    pub fn expr(&self) -> &LinearExpr<VariableKey> {
        &self.expression
    }
}

impl fmt::Display for Objective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sense_str = match self.sense {
            ObjectiveSense::Minimize => "Minimize",
            ObjectiveSense::Maximize => "Maximize",
        };

        write!(f, "{} {}", sense_str, self.expression)
    }
}
