use std::fmt;
use slotmap::new_key_type;

use crate::{
    modeling::expression::{ExprVariable, impl_expr_display, impl_expr_ops},
    standard_form::variable::StandardVariableKey,
};

new_key_type! {
    pub struct DictionaryVariableKey;
}

impl fmt::Display for DictionaryVariableKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DictionaryVariableKey({:?})", self.0)
    }
}

impl ExprVariable for DictionaryVariableKey {}

impl_expr_display!(DictionaryVariableKey);
impl_expr_ops!(DictionaryVariableKey, [f64]);

#[derive(Debug, Clone)]
pub enum DictionaryVariable {
    NonSlack(StandardVariableKey),
    Slack(usize),
    Auxiliary,
}

impl DictionaryVariable {
    pub fn new_non_slack(var: StandardVariableKey) -> Self {
        DictionaryVariable::NonSlack(var)
    }

    pub fn new_slack(index: usize) -> Self {
        DictionaryVariable::Slack(index)
    }

    pub fn new_auxiliary() -> Self {
        DictionaryVariable::Auxiliary
    }
}

impl fmt::Display for DictionaryVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DictionaryVariable::NonSlack(var) => write!(f, "DictVar({})", var),
            DictionaryVariable::Slack(index) => write!(f, "DictVar(Slack_{})", index),
            DictionaryVariable::Auxiliary => write!(f, "DictVar(Aux)"),
        }
    }
}
