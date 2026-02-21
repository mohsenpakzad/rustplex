use std::fmt;
use slotmap::{new_key_type, DenseSlotMap};

use crate::core::expression::{impl_expr_display, impl_expr_ops, ExprVariable};

new_key_type! {
    pub struct StandardVariableKey;
}

impl fmt::Display for StandardVariableKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StandardVariableKey({:?})", self.0)
    }
}

impl ExprVariable for StandardVariableKey {}

impl_expr_display!(StandardVariableKey);
impl_expr_ops!(StandardVariableKey, [f64, i32]);

#[derive(Debug, Clone)]
pub struct StandardVariable {
    name: Option<String>,
}

// Public Getters for Read-Only Access
impl StandardVariable {
    pub fn new() -> Self {
        Self {
            name: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    } 

    /// Returns the name of the standard variable.
    pub fn name(&self) -> String {
        if let Some(name) = &self.name {
            name.clone()
        } else {
            "<unnamed>".to_string()
        }
    }
}

impl Default for StandardVariable {
    fn default() -> Self {
        Self {
            name: None,
        }
    }
}

impl fmt::Display for StandardVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StandardVariable({})",
            self.name()
        )
    }
}

// --- Standard Variable Builder ---

/// A builder for creating and configuring a new standard variable.
pub struct StandardVariableBuilder<'a> {
    arena: &'a mut DenseSlotMap<StandardVariableKey, StandardVariable>,
    data: StandardVariable,
}

impl<'a> StandardVariableBuilder<'a> {
    pub(crate) fn new(arena: &'a mut DenseSlotMap<StandardVariableKey, StandardVariable>) -> Self {
        Self {
            arena,
            data: StandardVariable::default(),
        }
    }

    /// Sets the name of the standard variable.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.data.name = Some(name.into());
        self
    }

    // --- Terminating Methods ---

    /// Finalizes the variable creation.
    ///
    /// Since Standard Variables are always continuous, this method acts as the terminator.
    pub fn continuous(self) -> StandardVariableKey {
        self.finish()
    }

    /// Alias for `continuous()`.
    pub fn real(self) -> StandardVariableKey {
        self.continuous()
    }

    fn finish(self) -> StandardVariableKey {
        self.arena.insert(self.data)
    }
}
