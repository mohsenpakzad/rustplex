use std::{
    fmt,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::{
    core::expression::{impl_expr_display, impl_expr_ops, ExprVariable},
    standardization::standard_variable::StdVar,
};

#[derive(Debug, Clone)]
pub enum DictVariable {
    NonSlack(StdVar),
    Slack(usize),
}

#[derive(Debug, Clone)]
pub struct DictVar(Rc<DictVariable>);

impl DictVar {
    pub fn new_non_slack(var: StdVar) -> Self {
        DictVar(Rc::new(DictVariable::NonSlack(var)))
    }

    pub fn new_slack(idx: usize) -> Self {
        DictVar(Rc::new(DictVariable::Slack(idx)))
    }

    pub fn var(&self) -> &DictVariable {
        &self.0
    }
}

impl PartialEq for DictVar {
    fn eq(&self, other: &Self) -> bool {
        match (self.var(), other.var()) {
            (DictVariable::NonSlack(a), DictVariable::NonSlack(b)) => a == b,
            (DictVariable::Slack(a), DictVariable::Slack(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for DictVar {}

impl Hash for DictVar {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.var() {
            DictVariable::NonSlack(var) => {
                0.hash(state); // Discriminant for NonSlack variant
                var.hash(state);
            }
            DictVariable::Slack(idx) => {
                1.hash(state); // Discriminant for Slack variant
                idx.hash(state);
            }
        }
    }
}

impl fmt::Display for DictVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.var() {
            DictVariable::NonSlack(var) => write!(f, "DictVar({})", var),
            DictVariable::Slack(idx) => write!(f, "DictVar(Slack_{})", idx),
        }
    }
}

impl ExprVariable for DictVar {}

impl_expr_display!(DictVar);
impl_expr_ops!(DictVar, [f64]);
