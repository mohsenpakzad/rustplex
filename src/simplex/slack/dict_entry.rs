use std::{fmt, mem};

use super::dict_variable::DictVarRef;
use crate::core::expression::LinearExpr;

#[derive(Debug, Clone)]
pub struct DictEntry {
    basic_var: DictVarRef,
    non_basics_expr: LinearExpr<DictVarRef>,
}

impl DictEntry {
    /// Creates a new dictionary entry with a basic variable and an expression for non-basic variables.
    pub fn new(basic_var: DictVarRef, non_basics_expr: LinearExpr<DictVarRef>) -> Self {
        Self {
            basic_var,
            non_basics_expr,
        }
    }

    /// Adds a non-basic variable with a given coefficient to the expression.
    pub fn add_non_basic(&mut self, var: DictVarRef, coefficient: f64) {
        self.non_basics_expr.add_term(var, coefficient);
    }

    /// Checks if a given variable is present in the non-basic expression.
    pub fn contains_non_basic(&self, var: &DictVarRef) -> bool {
        self.non_basics_expr.terms.contains_key(var)
    }

    /// Removes a non-basic variable from the expression and
    /// returns its coefficient if it existed.
    pub fn remove_non_basic(&mut self, var: DictVarRef) -> Option<f64> {
        self.non_basics_expr.remove_term(&var)
    }

    /// Replaces a non-basic variable with an expression,
    /// scaling the new expression by the old variable's coefficient.
    pub fn replace_non_basic_with_expr(
        &mut self,
        var: DictVarRef,
        replacement_expr: &LinearExpr<DictVarRef>,
    ) -> Option<f64> {
        self.non_basics_expr
            .replace_var_with_expr(var, replacement_expr)
    }

    /// Switches the given non-basic variable to a basic variable,
    /// scaling the expression and setting the old basic variable as non-basic.
    pub fn switch_to_basic(&mut self, non_basic_var: DictVarRef) -> Option<f64> {
        if let Some(coefficient) = self.non_basics_expr.remove_term(&non_basic_var) {
            let old_basic_var = mem::replace(&mut self.basic_var, non_basic_var);

            self.non_basics_expr.add_term(old_basic_var, -1.0);
            self.non_basics_expr.scale(1.0 / -coefficient);
            Some(coefficient)
        } else {
            None
        }
    }
    /// Retrieves the coefficient of a non-basic variable from the non-basic expression.
    pub fn get_non_basic_coefficient(&self, var: &DictVarRef) -> f64 {
        self.non_basics_expr.get_coefficient(var)
    }

    pub fn get_basic_var(&self) -> DictVarRef {
        self.basic_var.clone()
    }

    pub fn get_value(&self) -> f64 {
        self.non_basics_expr.constant
    }

    pub fn get_expr(&self) -> &LinearExpr<DictVarRef> {
        &self.non_basics_expr
    }
}

impl fmt::Display for DictEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.basic_var, self.non_basics_expr)
    }
}
