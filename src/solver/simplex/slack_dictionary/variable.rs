use crate::{
    common::expression::{impl_expr_display, impl_expr_ops, ExprVariable},
    standard_form::variable::StandardVariableKey,
};
use slotmap::new_key_type;
use std::fmt;

new_key_type! {
    pub struct DictionaryVariableKey;
}

impl fmt::Display for DictionaryVariableKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DictionaryVariableKey({:?})", self.0)
    }
}

impl ExprVariable for DictionaryVariableKey {}

impl_expr_ops!(DictionaryVariableKey);
impl_expr_display!(DictionaryVariableKey);

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
