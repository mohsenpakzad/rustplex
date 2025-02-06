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
pub struct DictVarRef(Rc<DictVariable>);

impl DictVarRef {
    pub fn new_non_slack(var: StdVar) -> Self {
        DictVarRef(Rc::new(DictVariable::NonSlack(var)))
    }

    pub fn new_slack(idx: usize) -> Self {
        DictVarRef(Rc::new(DictVariable::Slack(idx)))
    }

    pub fn get_var(&self) -> &DictVariable {
        &self.0
    }
}

impl PartialEq for DictVarRef {
    fn eq(&self, other: &Self) -> bool {
        match (self.get_var(), other.get_var()) {
            (DictVariable::NonSlack(a), DictVariable::NonSlack(b)) => a == b,
            (DictVariable::Slack(a), DictVariable::Slack(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for DictVarRef {}

impl Hash for DictVarRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.get_var() {
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

impl fmt::Display for DictVarRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get_var() {
            DictVariable::NonSlack(var) => write!(f, "DictVar({})", var),
            DictVariable::Slack(idx) => write!(f, "DictVar(Slack_{})", idx),
        }
    }
}

impl ExprVariable for DictVarRef {}

impl_expr_display!(DictVarRef);
impl_expr_ops!(DictVarRef, [f64]);
