use std::{cell::RefCell, fmt, rc::Rc};

use crate::core::expression::LinearExpr;

use super::variable::VarRef;

#[derive(Debug, Clone)]
struct Constraint {
    name: Option<String>,
    lhs: LinearExpr<VarRef>,
    sense: ConstraintSense,
    rhs: LinearExpr<VarRef>,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstraintSense {
    LessEqual,
    GreaterEqual,
    Equal,
}

#[derive(Debug, Clone)]
pub struct ConstrRef(Rc<RefCell<Constraint>>);

impl ConstrRef {
    pub fn new(lhs: LinearExpr<VarRef>, sense: ConstraintSense, rhs: LinearExpr<VarRef>) -> Self {
        Self(Rc::new(RefCell::new(Constraint {
            name: None,
            lhs,
            sense,
            rhs,
        })))
    }

    pub fn name(self, name: impl Into<String>) -> Self {
        self.0.borrow_mut().name = Some(name.into());
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

    pub fn get_sense(&self) -> ConstraintSense {
        self.0.borrow().sense.clone()
    }

    pub fn get_lhs(&self) -> LinearExpr<VarRef> {
        self.0.borrow().lhs.clone()
    }

    pub fn get_rhs(&self) -> LinearExpr<VarRef> {
        self.0.borrow().rhs.clone()
    }
}

impl fmt::Display for ConstrRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sense = match &self.0.borrow().sense {
            ConstraintSense::LessEqual => "<=",
            ConstraintSense::GreaterEqual => ">=",
            ConstraintSense::Equal => "=",
        };
        write!(
            f,
            "{} {} {}",
            &self.0.borrow().lhs,
            sense,
            &self.0.borrow().rhs
        )
    }
}
