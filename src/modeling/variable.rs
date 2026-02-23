use std::fmt;
use std::ops::RangeInclusive;
use slotmap::{new_key_type, DenseSlotMap};

use crate::common::expression::{impl_expr_display, impl_expr_ops, ExprVariable};

new_key_type! {
    pub struct VariableKey;
}

impl fmt::Display for VariableKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Displays the internal ID (e.g., "Var(1v1)")
        write!(f, "VariableKey({:?})", self.0)
    }
}

impl ExprVariable for VariableKey {}

impl_expr_display!(VariableKey);
impl_expr_ops!(VariableKey, [f64, i32]);

#[derive(Debug, Clone, Copy)]
pub enum VariableType {
    Continuous,
    Integer,
    Binary,
}

#[derive(Debug)]
pub struct Variable {
    name: Option<String>,
    var_type: VariableType,
    lower_bound: f64,
    upper_bound: f64,
}

// Public Getters for Read-Only Access
impl Variable {
    /// Returns the name of the variable.
    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("<unnamed>")
    }

    /// Returns the type of the variable.
    pub fn var_type(&self) -> VariableType {
        self.var_type
    }

    /// Returns the lower bound of the variable.
    pub fn lower_bound(&self) -> f64 {
        self.lower_bound
    }

    /// Returns the upper bound of the variable.
    pub fn upper_bound(&self) -> f64 {
        self.upper_bound
    }
}

impl Default for Variable {
    fn default() -> Self {
        Self {
            name: None,
            var_type: VariableType::Continuous,
            lower_bound: f64::NEG_INFINITY,
            upper_bound: f64::INFINITY,
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self.var_type {
            VariableType::Continuous => "cont",
            VariableType::Integer => "int",
            VariableType::Binary => "bin",
        };

        write!(
            f,
            "Variable({}:{} ∈ [{}, {}])",
            self.name(),
            type_str,
            self.lower_bound,
            self.upper_bound
        )
    }
}

// --- Variable Builder ---

/// A builder for creating and configuring a new variable.
pub struct VariableBuilder<'a> {
    arena: &'a mut DenseSlotMap<VariableKey, Variable>,
    data: Variable,
}

impl<'a> VariableBuilder<'a> {
    pub(crate) fn new(arena: &'a mut DenseSlotMap<VariableKey, Variable>) -> Self {
        Self {
            arena,
            data: Variable::default(),
        }
    }

    /// Sets the name of the variable.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.data.name = Some(name.into());
        self
    }

    /// Sets the lower bound of the variable.
    pub fn lower_bound(mut self, lb: f64) -> Self {
        self.data.lower_bound = lb;
        self
    }

    /// Sets the upper bound of the variable.
    pub fn upper_bound(mut self, ub: f64) -> Self {
        self.data.upper_bound = ub;
        self
    }

    /// Convenience method for setting bounds using a Range.
    /// Example: `.bounds(0.0..=10.0)`
    pub fn bounds(mut self, range: RangeInclusive<f64>) -> Self {
        self.data.lower_bound = *range.start();
        self.data.upper_bound = *range.end();
        self
    }

    // --- Terminating Methods ---

    /// Finalizes the variable as **Continuous**.
    pub fn continuous(self) -> VariableKey {
        self.finish(VariableType::Continuous)
    }

    /// Alias for `continuous()`.
    pub fn real(self) -> VariableKey {
        self.continuous()
    }

    /// Finalizes the variable as **Integer**.
    pub fn integer(self) -> VariableKey {
        self.finish(VariableType::Integer)
    }

    /// Finalizes the variable as **Binary**.
    ///
    /// This automatically sets the bounds to [0.0, 1.0].
    pub fn binary(mut self) -> VariableKey {
        self.data.lower_bound = 0.0;
        self.data.upper_bound = 1.0;
        self.finish(VariableType::Binary)
    }

    fn finish(mut self, var_type: VariableType) -> VariableKey {
        self.data.var_type = var_type;
        self.arena.insert(self.data)
    }
}