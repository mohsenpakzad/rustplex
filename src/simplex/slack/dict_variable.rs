use std::fmt;
use slotmap::new_key_type;

use crate::{
    core::expression::{ExprVariable, impl_expr_display, impl_expr_ops},
    standard::variable::StandardVariableKey,
};

new_key_type! {
    pub struct DictVariableKey;
}

impl fmt::Display for DictVariableKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DictVariableKey({:?})", self.0)
    }
}

impl ExprVariable for DictVariableKey {}

impl_expr_display!(DictVariableKey);
impl_expr_ops!(DictVariableKey, [f64]);

#[derive(Debug, Clone)]
pub enum DictVariable {
    NonSlack(StandardVariableKey),
    Slack(usize),
    Auxiliary,
}

impl DictVariable {
    pub fn new_non_slack(var: StandardVariableKey) -> Self {
        DictVariable::NonSlack(var)
    }

    pub fn new_slack(index: usize) -> Self {
        DictVariable::Slack(index)
    }

    pub fn new_auxiliary() -> Self {
        DictVariable::Auxiliary
    }
}

impl fmt::Display for DictVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DictVariable::NonSlack(var) => write!(f, "DictVar({})", var),
            DictVariable::Slack(index) => write!(f, "DictVar(Slack_{})", index),
            DictVariable::Auxiliary => write!(f, "DictVar(Aux)"),
        }
    }
}
