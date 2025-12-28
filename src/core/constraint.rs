use std::{cell::RefCell, fmt, rc::Rc};

use crate::core::expression::LinearExpr;

use super::variable::Var;

#[derive(Debug, Clone)]
struct Constraint {
    name: Option<String>,
    lhs: LinearExpr<Var>,
    sense: ConstraintSense,
    rhs: LinearExpr<Var>,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstraintSense {
    LessEqual,
    GreaterEqual,
    Equal,
}

#[derive(Debug, Clone)]
pub struct Constr(Rc<RefCell<Constraint>>);

impl Constr {
    pub fn new(
        lhs: impl Into<LinearExpr<Var>>,
        sense: ConstraintSense,
        rhs: impl Into<LinearExpr<Var>>,
    ) -> Self {
        Self(Rc::new(RefCell::new(Constraint {
            name: None,
            lhs: lhs.into(),
            sense,
            rhs: rhs.into(),
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

    pub fn sense(&self) -> ConstraintSense {
        self.0.borrow().sense.clone()
    }

    pub fn lhs(&self) -> LinearExpr<Var> {
        self.0.borrow().lhs.clone()
    }

    pub fn rhs(&self) -> LinearExpr<Var> {
        self.0.borrow().rhs.clone()
    }
}

impl fmt::Display for Constr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name_display = match self.name() {
            Some(name) => name.clone(),
            None => self.name_or_default(),
        };

        let sense = match self.sense() {
            ConstraintSense::LessEqual => "<=",
            ConstraintSense::GreaterEqual => ">=",
            ConstraintSense::Equal => "=",
        };

        write!(
            f,
            "Constr({}): {} {} {}",
            name_display,
            self.lhs(),
            sense,
            self.rhs(),
        )
    }
}
