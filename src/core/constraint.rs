use std::fmt;
use slotmap::{new_key_type, DenseSlotMap};

use crate::core::expression::LinearExpr;
use crate::core::variable::VariableKey;

new_key_type! {
    pub struct ConstraintKey;
}

impl fmt::Display for ConstraintKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConstraintKey({:?})", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConstraintSense {
    LessEqual,
    GreaterEqual,
    Equal,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    name: Option<String>,
    lhs: LinearExpr<VariableKey>,
    sense: ConstraintSense,
    rhs: LinearExpr<VariableKey>,
}

// Public Getters for Read-Only Access
impl Constraint {
    /// Returns the name of the constraint.
    pub fn name(&self) -> String {
        if let Some(name) = &self.name {
            name.clone()
        } else {
            "<unnamed>".to_string()
        }
    }

    /// Returns the Left Hand Side expression.
    pub fn lhs(&self) -> &LinearExpr<VariableKey> {
        &self.lhs
    }

    /// Returns the Right Hand Side expression.
    pub fn rhs(&self) -> &LinearExpr<VariableKey> {
        &self.rhs
    }

    /// Returns the comparison sense (<=, >=, =).
    pub fn sense(&self) -> ConstraintSense {
        self.sense
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sense_str = match self.sense {
            ConstraintSense::LessEqual => "<=",
            ConstraintSense::GreaterEqual => ">=",
            ConstraintSense::Equal => "=",
        };

        write!(
            f,
            "Constraint({}: {} {} {})",
            self.name(),
            self.lhs,
            sense_str,
            self.rhs
        )
    }
}

// --- Constraint Builder ---

/// A builder for creating and configuring a new constraint.
pub struct ConstraintBuilder<'a> {
    arena: &'a mut DenseSlotMap<ConstraintKey, Constraint>,
    lhs: LinearExpr<VariableKey>,
    name: Option<String>,
}

impl<'a> ConstraintBuilder<'a> {
    pub(crate) fn new(
        arena: &'a mut DenseSlotMap<ConstraintKey, Constraint>,
        lhs: LinearExpr<VariableKey>,
    ) -> Self {
        Self {
            arena,
            lhs,
            name: None,
        }
    }

    /// Sets the name of the constraint.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    // --- Terminating Methods ---

    /// Creates a Less Than or Equal constraint: `LHS <= RHS`.
    pub fn less_than_or_equal(self, rhs: impl Into<LinearExpr<VariableKey>>) -> ConstraintKey {
        self.finish(ConstraintSense::LessEqual, rhs.into())
    }

    /// Alias for `less_than_or_equal`.
    pub fn le(self, rhs: impl Into<LinearExpr<VariableKey>>) -> ConstraintKey {
        self.less_than_or_equal(rhs)
    }

    /// Creates a Greater Than or Equal constraint: `LHS >= RHS`.
    pub fn greater_than_or_equal(self, rhs: impl Into<LinearExpr<VariableKey>>) -> ConstraintKey {
        self.finish(ConstraintSense::GreaterEqual, rhs.into())
    }

    /// Alias for `greater_than_or_equal`.
    pub fn ge(self, rhs: impl Into<LinearExpr<VariableKey>>) -> ConstraintKey {
        self.greater_than_or_equal(rhs)
    }

    /// Creates an Equality constraint: `LHS == RHS`.
    pub fn equal_to(self, rhs: impl Into<LinearExpr<VariableKey>>) -> ConstraintKey {
        self.finish(ConstraintSense::Equal, rhs.into())
    }

    /// Alias for `equal_to`.
    pub fn eq(self, rhs: impl Into<LinearExpr<VariableKey>>) -> ConstraintKey {
        self.equal_to(rhs)
    }

    fn finish(self, sense: ConstraintSense, rhs: LinearExpr<VariableKey>) -> ConstraintKey {
        let data = Constraint {
            name: self.name,
            lhs: self.lhs,
            sense,
            rhs,
        };
        self.arena.insert(data)
    }
}