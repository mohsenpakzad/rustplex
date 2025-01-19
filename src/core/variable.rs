use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::RangeInclusive;
use std::rc::Rc;

use super::expression::{impl_expr_ops, ExprVariable};

#[derive(Debug)]
struct Variable {
    name: Option<String>,
    var_type: VariableType,
    bounds: RangeInclusive<f64>,
}

#[derive(Debug, Clone)]
pub enum VariableType {
    Continuous,
    Integer,
    Binary,
}

impl Default for Variable {
    fn default() -> Self {
        Self {
            name: None,
            var_type: VariableType::Continuous,
            bounds: f64::NEG_INFINITY..=f64::INFINITY,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VarRef(Rc<RefCell<Variable>>);

impl VarRef {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(self, name: impl Into<String>) -> Self {
        self.0.borrow_mut().name = Some(name.into());
        self
    }

    pub fn continuous(self) -> Self {
        self.0.borrow_mut().var_type = VariableType::Continuous;
        self
    }

    pub fn integer(self) -> Self {
        self.0.borrow_mut().var_type = VariableType::Integer;
        self
    }

    pub fn binary(self) -> Self {
        {
            let mut var = self.0.borrow_mut();
            var.var_type = VariableType::Binary;
            var.bounds = 0.0..=1.0;
        }
        self
    }

    pub fn bounds(self, bounds: RangeInclusive<f64>) -> Self {
        self.0.borrow_mut().bounds = bounds;
        self
    }

    pub fn lower_bound(self, lb: f64) -> Self {
        {
            let mut var: std::cell::RefMut<'_, Variable> = self.0.borrow_mut();
            var.bounds = lb..=*var.bounds.end()
        }
        self
    }

    pub fn upper_bound(self, ub: f64) -> Self {
        {
            let mut var: std::cell::RefMut<'_, Variable> = self.0.borrow_mut();
            var.bounds = *var.bounds.start()..=ub;
        }
        self
    }

    pub fn get_name(&self) -> Option<String> {
        self.0.borrow().name.clone()
    }

    pub fn get_name_or_default(&self) -> String {
        self.0
            .borrow()
            .name
            .clone()
            .unwrap_or(format!("{:p}", Rc::as_ptr(&self.0)))
    }

    pub fn get_type(&self) -> VariableType {
        self.0.borrow().var_type.clone()
    }

    pub fn get_lower_bound(&self) -> f64 {
        *self.0.borrow().bounds.start()
    }

    pub fn get_upper_bound(&self) -> f64 {
        *self.0.borrow().bounds.end()
    }
}

impl PartialEq for VarRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for VarRef {}

impl Hash for VarRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Rc::as_ptr(&self.0) as usize).hash(state);
    }
}

impl fmt::Display for VarRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variable = self.0.borrow();
        match &variable.name {
            Some(name) => write!(f, "{}", name),
            None => write!(f, "var_{:p}", Rc::as_ptr(&self.0)),
        }
    }
}

impl ExprVariable for VarRef {}

impl_expr_ops!(VarRef, [f64, f32, i8, i16, i32, i64, i128, isize]);
