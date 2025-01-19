use std::{fmt, rc::Rc};

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
pub struct ConstrRef(Rc<Constraint>);

impl ConstrRef {
    pub fn new(lhs: LinearExpr<VarRef>, sense: ConstraintSense, rhs: LinearExpr<VarRef>) -> Self {
        Self(Rc::new(Constraint {
            name: None,
            lhs,
            sense,
            rhs,
        }))
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.constraint().name = Some(name.into());
        self
    }

    pub fn get_sense(&self) -> ConstraintSense {
        self.0.sense
    }

    pub fn get_lhs(&self) -> &LinearExpr<VarRef> {
        &self.0.lhs
    }

    pub fn get_rhs(&self) -> &LinearExpr<VarRef> {
        &self.0.rhs
    }

    fn constraint(&mut self) -> &mut Constraint {
        Rc::make_mut(&mut self.0)
    }
}

impl fmt::Display for ConstrRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sense = match &self.0.sense {
            ConstraintSense::LessEqual => "<=",
            ConstraintSense::GreaterEqual => ">=",
            ConstraintSense::Equal => "=",
        };
        write!(f, "{} {} {}", &self.0.lhs, sense, &self.0.rhs)
    }
}
