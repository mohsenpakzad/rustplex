use crate::core::expression::{impl_expr_display, impl_expr_ops, ExprVariable};
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::{fmt, rc::Rc};

#[derive(Debug, Clone)]
pub struct StandardVariable {
    name: Option<String>,
    shift: f64,
    upper_bound: f64,
    is_negative_part: bool,
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

#[derive(Debug, Clone, Default)]
pub struct StdVarRef(Rc<RefCell<StandardVariable>>);

impl StdVarRef {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_positive() -> Self {
        Self::default()
    }

    pub fn new_negative() -> Self {
        Self(Rc::new(RefCell::new(StandardVariable {
            name: None,
            shift: 0.0,
            upper_bound: f64::INFINITY,
            is_negative_part: true,
        })))
    }

    pub fn name(self, name: impl Into<String>) -> Self {
        self.0.borrow_mut().name = Some(name.into());
        self
    }

    pub fn shift(self, shift: f64) -> Self {
        self.0.borrow_mut().shift = shift;
        self
    }

    pub fn upper_bound(self, ub: f64) -> Self {
        self.0.borrow_mut().upper_bound = ub;
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

    pub fn is_negative_part(&self) -> bool {
        self.0.borrow().is_negative_part
    }

    pub fn get_shift_value(&self) -> f64 {
        self.0.borrow().shift
    }

    pub fn get_upper_bound(&self) -> f64 {
        self.0.borrow().upper_bound
    }
}

impl PartialEq for StdVarRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for StdVarRef {}

impl Hash for StdVarRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Rc::as_ptr(&self.0) as usize).hash(state);
    }
}

impl fmt::Display for StdVarRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name_display = match self.get_name() {
            Some(name) => name.clone(),
            None => self.get_name_or_default(),
        };

        let sign = if self.is_negative_part() {
            "⁻"
        } else {
            "⁺"
        };

        write!(f, "StdVar({}){}", name_display, sign)
    }
}

impl ExprVariable for StdVarRef {}

impl_expr_display!(StdVarRef);
impl_expr_ops!(StdVarRef, [f64, f32, i8, i16, i32, i64, i128, isize]);
