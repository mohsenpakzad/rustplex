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
    shift: f64,
    upper_bound: f64,
    is_negative_part: bool,
}

// Public Getters for Read-Only Access
impl StandardVariable {
    pub fn new_positive() -> Self {
        Self {
            name: None,
            shift: 0.0,
            upper_bound: f64::INFINITY,
            is_negative_part: false,
        }
    }

    pub fn new_negative() -> Self {
        Self {
            name: None,
            shift: 0.0,
            upper_bound: f64::INFINITY,
            is_negative_part: true,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    } 

    pub fn with_shift(mut self, shift: f64) -> Self {
        self.shift = shift;
        self
    }

    pub fn with_upper_bound(mut self, upper_bound: f64) -> Self {
        self.upper_bound = upper_bound;
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

    /// Returns the shift of the standard variable.
    pub fn shift(&self) -> f64 {
        self.shift
    }

    /// Returns the upper bound of the standard variable.
    pub fn upper_bound(&self) -> f64 {
        self.upper_bound
    }

    /// Returns if the standard variable is negative part.
    pub fn is_negative_part(&self) -> bool {
        self.is_negative_part
    }
}

impl Default for StandardVariable {
    fn default() -> Self {
        Self {
            name: None,
            shift: 0.0,
            upper_bound: f64::INFINITY,
            is_negative_part: false,
        }
    }
}

impl fmt::Display for StandardVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.is_negative_part {
            "⁻"
        } else {
            "⁺"
        };

        let ub_str = if self.upper_bound.is_infinite() {
            "inf".to_string()
        } else {
            self.upper_bound.to_string()
        };

        write!(
            f,
            "StandardVariable({}{}; shift={}, ub={})",
            self.name(),
            sign,
            self.shift,
            ub_str
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

    /// Sets the shift of the standard variable.
    pub fn shift(mut self, shift: f64) -> Self {
        self.data.shift = shift;
        self
    }

    /// Sets the upper bound of the standard variable.
    pub fn upper_bound(mut self, upper_bound: f64) -> Self {
        self.data.upper_bound = upper_bound;
        self
    }

    /// Sets the standard variable to be positive part.
    pub fn positive(mut self) -> Self{
        self.data.is_negative_part = false;
        self
    }

    /// Sets the standard variable to be negative part.
    pub fn negative(mut self) -> Self{
        self.data.is_negative_part = true;
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
