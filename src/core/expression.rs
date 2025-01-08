use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::core::variable::VarRef;

#[derive(Debug, Clone)]
pub struct LinearExpr {
    pub terms: HashMap<VarRef, f64>,
    pub constant: f64,
}

impl LinearExpr {
    pub fn new() -> Self {
        Self {
            terms: HashMap::new(),
            constant: 0.0,
        }
    }

    pub fn with_terms(terms: HashMap<VarRef, f64>) -> Self {
        Self {
            terms,
            constant: 0.0,
        }
    }

    pub fn with_constant(constant: f64) -> Self {
        Self {
            terms: HashMap::new(),
            constant,
        }
    }

    pub fn with_terms_and_constant(terms: HashMap<VarRef, f64>, constant: f64) -> Self {
        Self { terms, constant }
    }

    pub fn add_term(&mut self, var: VarRef, coefficient: f64) {
        *self.terms.entry(var).or_insert(0.0) += coefficient;
    }

    pub fn get_coefficient(&self, var: &VarRef) -> f64 {
        *self.terms.get(var).unwrap_or(&0.0)
    }

    pub fn scale(&mut self, scalar: f64) {
        for coeff in self.terms.values_mut() {
            *coeff *= scalar;
        }
        self.constant *= scalar;
    }
}

impl fmt::Display for LinearExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for (var, &coeff) in &self.terms {
            if !first && coeff >= 0.0 {
                write!(f, " + ")?;
            }
            if coeff == -1.0 {
                write!(f, " - ")?;
            } else if coeff != 1.0 {
                write!(f, "{}", coeff)?;
            }
            write!(f, "{}", var)?;
            first = false;
        }
        if self.constant != 0.0 || first {
            if !first && self.constant >= 0.0 {
                write!(f, " + ")?;
            }
            write!(f, "{}", self.constant)?;
        }
        Ok(())
    }
}

// Implement From<VarRef> for LinearExpr to handle direct conversion
impl From<VarRef> for LinearExpr {
    fn from(var: VarRef) -> Self {
        let mut terms = HashMap::new();
        terms.insert(var, 1.0);
        LinearExpr::with_terms(terms)
    }
}

// Implement Add for VarRef + VarRef
impl Add for VarRef {
    type Output = LinearExpr;

    fn add(self, other: Self) -> LinearExpr {
        let mut terms = HashMap::new();
        terms.insert(self, 1.0);
        terms.insert(other, 1.0);
        LinearExpr::with_terms(terms)
    }
}

// VarRef + LinearExpr
impl Add<LinearExpr> for VarRef {
    type Output = LinearExpr;

    fn add(self, mut expr: LinearExpr) -> LinearExpr {
        expr.add_term(self, 1.0);
        expr
    }
}

// LinearExpr + VarRef
impl Add<VarRef> for LinearExpr {
    type Output = LinearExpr;

    fn add(mut self, var: VarRef) -> LinearExpr {
        self.add_term(var, 1.0);
        self
    }
}

// Implement Add for LinearExpr + LinearExpr
impl Add for LinearExpr {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        for (var, coeff) in other.terms {
            *self.terms.entry(var).or_insert(0.0) += coeff;
        }
        self.constant += other.constant;
        self
    }
}

// Implement Add for LinearExpr - LinearExpr
impl Sub for LinearExpr {
    type Output = Self;

    fn sub(mut self, other: Self) -> Self {
        for (var, coeff) in other.terms {
            *self.terms.entry(var).or_insert(0.0) -= coeff;
        }
        self.constant -= other.constant;
        self
    }
}

// Implement Neg for - LinearExpr
impl Neg for LinearExpr {
    type Output = Self;

    fn neg(mut self) -> Self {
        for coeff in self.terms.values_mut() {
            *coeff = -*coeff;
        }
        self.constant = -self.constant;
        self
    }
}

macro_rules! impl_ops {
    ($($type:ty),*) => {
        $(
            impl From<$type> for LinearExpr {
                fn from(constant: $type) -> Self {
                    LinearExpr::with_constant(constant as f64)
                }
            }

            // Implement Add for VarRef + $type
            impl Add<$type> for VarRef {
                type Output = LinearExpr;

                fn add(self, constant: $type) -> LinearExpr {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0);
                    LinearExpr::with_terms_and_constant(terms, constant as f64)
                }
            }

            // Implement Add for $type + VarRef
            impl Add<VarRef> for $type {
                type Output = LinearExpr;

                fn add(self, var: VarRef) -> LinearExpr {
                    var + self
                }
            }

            // Implement Sub for VarRef - $type
            impl Sub<$type> for VarRef {
                type Output = LinearExpr;

                fn sub(self, constant: $type) -> LinearExpr {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0);
                    LinearExpr::with_terms_and_constant(terms, -constant as f64)
                }
            }

            // Implement Sub for $type - VarRef
            impl Sub<VarRef> for $type {
                type Output = LinearExpr;

                fn sub(self, var: VarRef) -> LinearExpr {
                    var - self
                }
            }

            // Implement Mul for VarRef * $type
            impl Mul<$type> for VarRef {
                type Output = LinearExpr;

                fn mul(self, coeff: $type) -> LinearExpr {
                    let mut terms = HashMap::new();
                    terms.insert(self, coeff as f64);
                    LinearExpr::with_terms(terms)
                }
            }

            // Implement Mul for $type * VarRef
            impl Mul<VarRef> for $type {
                type Output = LinearExpr;

                fn mul(self, var: VarRef) -> LinearExpr {
                    var * self
                }
            }

            // Implement Div for VarRef / $type
            impl Div<$type> for VarRef {
                type Output = LinearExpr;

                fn div(self, coeff: $type) -> LinearExpr {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0 / coeff as f64);
                    LinearExpr::with_terms(terms)
                }
            }

            // Implement Div for $type / LinearExpr
            impl Div<VarRef> for $type {
                type Output = LinearExpr;

                fn div(self, var: VarRef) -> LinearExpr {
                    var / self
                }
            }

            // Implement Add for LinearExpr + $type
            impl Add<$type> for LinearExpr {
                type Output = Self;

                fn add(mut self, constant: $type) -> Self {
                    self.constant += constant as f64;
                    self
                }
            }

            // Implement Add for $type + LinearExpr
            impl Add<LinearExpr> for $type {
                type Output = LinearExpr;

                fn add(self, mut expr: LinearExpr) -> LinearExpr {
                    expr.constant += self as f64;
                    expr
                }
            }

            // Implement Sub for LinearExpr - $type
            impl Sub<$type> for LinearExpr {
                type Output = Self;

                fn sub(mut self, constant: $type) -> Self {
                    self.constant -= constant as f64;
                    self
                }
            }

            // Implement Sub for $type - LinearExpr
            impl Sub<LinearExpr> for $type {
                type Output = LinearExpr;

                fn sub(self, mut expr: LinearExpr) -> LinearExpr {
                    expr = -expr;
                    expr.constant += self as f64;
                    expr
                }
            }
        )*
    };
}

impl_ops!(f64, f32, i8, i16, i32, i64, i128, isize);
