use std::{fmt, rc::Rc};

use crate::core::expression::LinearExpr;

#[derive(Debug, Clone)]
struct Constraint {
    name: Option<String>,
    lhs: LinearExpr,
    sense: ConstraintSense,
    rhs: LinearExpr,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstraintSense {
    LessThan,
    GreaterThan,
    Equal,
}

#[derive(Debug, Clone)]
pub struct ConstrRef(Rc<Constraint>);

impl ConstrRef {
    pub fn new(lhs: LinearExpr, sense: ConstraintSense, rhs: LinearExpr) -> Self {
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

    fn constraint(&mut self) -> &mut Constraint {
        Rc::make_mut(&mut self.0)
    }
}

impl fmt::Display for ConstrRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sense = match &self.0.sense {
            ConstraintSense::LessThan => "<=",
            ConstraintSense::GreaterThan => ">=",
            ConstraintSense::Equal => "=",
        };
        write!(f, "{} {} {}", &self.0.lhs, sense, &self.0.rhs)
    }
}
