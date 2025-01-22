use std::{cell::RefCell, fmt, mem, rc::Rc};

use super::dict_variable::DictVarRef;
use crate::core::expression::LinearExpr;

#[derive(Debug, Clone)]
pub struct DictEntry {
    basic_var: DictVarRef,
    non_basics_expr: LinearExpr<DictVarRef>,
}

#[derive(Debug, Clone)]
pub struct DictEntryRef(Rc<RefCell<DictEntry>>);

impl DictEntryRef {
    /// Creates a new reference to a dictionary entry.
    pub fn new(basic_var: DictVarRef, non_basics_expr: LinearExpr<DictVarRef>) -> Self {
        DictEntryRef(Rc::new(RefCell::new(DictEntry {
            basic_var,
            non_basics_expr,
        })))
    }

    /// Adds a non-basic variable with a given coefficient to the expression.
    pub fn add_non_basic(&self, var: DictVarRef, coefficient: f64) {
        self.0
            .borrow_mut()
            .non_basics_expr
            .add_term(var, coefficient);
    }

    /// Checks if a given variable is present in the non-basic expression.
    pub fn contains_non_basic(&self, var: &DictVarRef) -> bool {
        self.0.borrow().non_basics_expr.terms.contains_key(var)
    }

    /// Removes a non-basic variable from the expression and
    /// returns its coefficient if it existed.
    pub fn remove_non_basic(&self, var: DictVarRef) -> Option<f64> {
        self.0.borrow_mut().non_basics_expr.remove_term(&var)
    }

    pub fn get_non_basics(&self) -> Vec<DictVarRef> {
        self.0
            .borrow()
            .non_basics_expr
            .terms
            .keys()
            .map(DictVarRef::clone)
            .collect()
    }

    /// Retrieves the coefficient of a non-basic variable from the non-basic expression.
    pub fn get_non_basic_coefficient(&self, var: &DictVarRef) -> f64 {
        self.0.borrow().non_basics_expr.get_coefficient(var)
    }

    /// Replaces a non-basic variable with an expression,
    /// scaling the new expression by the old variable's coefficient.
    pub fn replace_non_basic_with_expr(
        &self,
        var: DictVarRef,
        replacement_expr: &LinearExpr<DictVarRef>,
    ) -> Option<f64> {
        self.0
            .borrow_mut()
            .non_basics_expr
            .replace_var_with_expr(var, replacement_expr)
    }

    /// Switches the given non-basic variable to a basic variable,
    /// scaling the expression and setting the old basic variable as non-basic.
    pub fn switch_to_basic(&self, non_basic_var: DictVarRef) -> Option<f64> {
        let mut entry = self.0.borrow_mut();

        if let Some(coefficient) = entry.non_basics_expr.remove_term(&non_basic_var) {
            let old_basic_var = mem::replace(&mut entry.basic_var, non_basic_var);

            entry.non_basics_expr.add_term(old_basic_var, -1.0);
            entry.non_basics_expr.scale(1.0 / -coefficient);
            Some(coefficient)
        } else {
            None
        }
    }

    /// Gets the basic variable of the dictionary entry.
    pub fn get_basic_var(&self) -> DictVarRef {
        self.0.borrow().basic_var.clone()
    }

    /// Gets the value (constant) of the dictionary entry.
    pub fn get_value(&self) -> f64 {
        self.0.borrow().non_basics_expr.constant
    }

    /// Gets the expression of non-basic variables in the dictionary entry.
    pub fn get_expr(&self) -> LinearExpr<DictVarRef> {
        self.0.borrow().non_basics_expr.clone()
    }
}

impl fmt::Display for DictEntryRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = {}",
            self.0.borrow().basic_var,
            self.0.borrow().non_basics_expr
        )
    }
}
