use crate::common::expression::LinearExpr;
use crate::standard_form::variable::StandardVariableKey;
use slotmap::new_key_type;
use std::fmt;

new_key_type! {
    pub struct StandardConstraintKey;
}

impl fmt::Display for StandardConstraintKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StandardConstraintKey({:?})", self.0)
    }
}

#[derive(Debug)]
pub struct StandardConstraint {
    name: Option<String>,
    lhs: LinearExpr<StandardVariableKey>,
    rhs: f64,
}

// Public Getters for Read-Only Access
impl StandardConstraint {
    pub fn new(lhs: impl Into<LinearExpr<StandardVariableKey>>, rhs: f64) -> Self {
        Self {
            name: None,
            lhs: lhs.into(),
            rhs,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Returns the name of the constraint.
    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("<unnamed>")
    }

    /// Returns the Left Hand Side expression.
    pub fn lhs(&self) -> &LinearExpr<StandardVariableKey> {
        &self.lhs
    }

    /// Returns the Right Hand Side constant.
    pub fn rhs(&self) -> f64 {
        self.rhs
    }
}

impl fmt::Display for StandardConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StandardConstraint({}: {} <= {})",
            self.name(),
            self.lhs,
            self.rhs
        )
    }
}
