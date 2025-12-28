use crate::core::expression::LinearExpr;
use std::{cell::RefCell, fmt, rc::Rc};

use super::standard_variable::StdVar;

#[derive(Debug)]
struct StandardConstraint {
    name: Option<String>,
    lhs: LinearExpr<StdVar>,
    rhs: f64,
}

#[derive(Debug)]
pub struct StdConstr(Rc<RefCell<StandardConstraint>>);

impl StdConstr {
    pub fn new(lhs: impl Into<LinearExpr<StdVar>>, rhs: f64) -> Self {
        Self(Rc::new(RefCell::new(StandardConstraint {
            name: None,
            lhs: lhs.into(),
            rhs,
        })))
    }

    pub fn with_name(self, name: impl Into<String>) -> Self {
        self.0.borrow_mut().name = Some(name.into());
        self
    }

    pub fn name(&self) -> Option<String> {
        self.0.borrow().name.clone()
    }

    pub fn name_or_default(&self) -> String {
        self.0
            .borrow()
            .name
            .clone()
            .unwrap_or(format!("{:p}", Rc::as_ptr(&self.0)))
    }

    pub fn lhs(&self) -> LinearExpr<StdVar> {
        self.0.borrow().lhs.clone()
    }

    pub fn rhs(&self) -> f64 {
        self.0.borrow().rhs
    }
}

impl Clone for StdConstr {
    fn clone(&self) -> Self {
        StdConstr(Rc::clone(&self.0))
    }
}

impl fmt::Display for StdConstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name_display = match self.name() {
            Some(name) => name.clone(),
            None => self.name_or_default(),
        };

        write!(
            f,
            "StdConstr({}): {} <= {}",
            name_display,
            self.lhs(),
            self.rhs()
        )
    }
}
