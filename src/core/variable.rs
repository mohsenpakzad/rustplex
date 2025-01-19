use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Debug, Clone)]
struct Variable {
    name: Option<String>,
    var_type: VariableType,
    lower_bound: f64,
    upper_bound: f64,
}

#[derive(Debug, Clone)]
pub enum VariableType {
    Continuous,
    Integer,
    Binary,
}

impl Default for Variable {
    fn default() -> Self {
        Self {
            name: None,
            var_type: VariableType::Continuous,
            lower_bound: f64::NEG_INFINITY,
            upper_bound: f64::INFINITY,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VarRef(Rc<Variable>);

impl VarRef {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.variable().name = Some(name.into());
        self
    }

    pub fn continuous(mut self) -> Self {
        self.variable().var_type = VariableType::Continuous;
        self
    }

    pub fn integer(mut self) -> Self {
        self.variable().var_type = VariableType::Integer;
        self
    }

    pub fn binary(mut self) -> Self {
        let var = self.variable();
        var.var_type = VariableType::Binary;
        var.lower_bound = 0.0;
        var.upper_bound = 1.0;
        self
    }

    pub fn lower_bound(mut self, lb: f64) -> Self {
        self.variable().lower_bound = lb;
        self
    }

    pub fn upper_bound(mut self, ub: f64) -> Self {
        self.variable().upper_bound = ub;
        self
    }

    pub fn bounds(mut self, lb: f64, ub: f64) -> Self {
        self.variable().lower_bound = lb;
        self.variable().upper_bound = ub;
        self
    }

    fn variable(&mut self) -> &mut Variable {
        Rc::make_mut(&mut self.0)
    }
}

impl Default for VarRef {
    fn default() -> Self {
        Self(Rc::new(Variable::default()))
    }
}

impl PartialEq for VarRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for VarRef {}

impl Hash for VarRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Rc::as_ptr(&self.0) as usize).hash(state);
    }
}

impl fmt::Display for VarRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0.name {
            Some(name) => write!(f, "{}", name),
            None => write!(f, "var_{:p}", Rc::as_ptr(&self.0)),
        }
    }
}

impl ExprVariable for VarRef {}

impl_expr_ops!(VarRef, [f64, f32, i8, i16, i32, i64, i128, isize]);
