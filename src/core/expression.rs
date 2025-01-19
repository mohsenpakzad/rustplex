use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct LinearExpr<T: ExprVariable> {
    pub terms: HashMap<T, f64>,
    pub constant: f64,
}

pub trait ExprVariable: Clone + Hash + Eq + fmt::Display {}

impl<T: ExprVariable> LinearExpr<T> {
    pub fn new() -> Self {
        Self {
            terms: HashMap::new(),
            constant: 0.0,
        }
    }

    pub fn with_terms(terms: HashMap<T, f64>) -> Self {
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

    pub fn with_terms_and_constant(terms: HashMap<T, f64>, constant: f64) -> Self {
        Self { terms, constant }
    }

    pub fn get_coefficient(&self, var: &T) -> f64 {
        *self.terms.get(var).unwrap_or(&0.0)
    }

    pub fn add_term(&mut self, var: T, coefficient: f64) {
        *self.terms.entry(var).or_insert(0.0) += coefficient;
    }

    pub fn scale(&mut self, scalar: f64) {
        for coeff in self.terms.values_mut() {
            *coeff *= scalar;
        }
        self.constant *= scalar;
    }
}

macro_rules! impl_expr_ops {
    ($var_type:ty, [$($num_type:ty),* $(,)?]) => {
        use std::collections::HashMap;
        use std::ops::{Add, Div, Mul, Neg, Sub};
        use crate::core::expression::LinearExpr;


        impl fmt::Display for LinearExpr<$var_type> {
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

        // Implement From<ExprVariable> for LinearExpr
        impl From<$var_type> for LinearExpr<$var_type> {
            fn from(var: $var_type) -> Self {
                let mut terms = HashMap::new();
                terms.insert(var, 1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // ExprVariable + ExprVariable
        impl Add for $var_type {
            type Output = LinearExpr<$var_type>;

            fn add(self, other: Self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self, 1.0);
                terms.insert(other, 1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // ExprVariable - ExprVariable
        impl Sub for $var_type {
            type Output = LinearExpr<$var_type>;

            fn sub(self, other: Self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self, 1.0);
                terms.insert(other, -1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // -ExprVariable
        impl Neg for $var_type {
            type Output = LinearExpr<$var_type>;

            fn neg(self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self, -1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // ExprVariable + LinearExpr
        impl Add<LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;

            fn add(self, mut expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                expr.add_term(self, 1.0);
                expr
            }
        }

        // ExprVariable - LinearExpr
        impl Sub<LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;

            fn sub(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                -expr + self
            }
        }

        // LinearExpr + ExprVariable
        impl Add<$var_type> for LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn add(mut self, var: $var_type) -> LinearExpr<$var_type> {
                self.add_term(var, 1.0);
                self
            }
        }

        // LinearExpr - ExprVariable
        impl Sub<$var_type> for LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn sub(mut self, var: $var_type) -> LinearExpr<$var_type> {
                self.add_term(var, -1.0);
                self
            }
        }

        // LinearExpr + LinearExpr
        impl Add for LinearExpr<$var_type> {
            type Output = Self;

            fn add(mut self, other: Self) -> Self {
                for (var, coeff) in other.terms {
                    *self.terms.entry(var).or_insert(0.0) += coeff;
                }
                self.constant += other.constant;
                self
            }
        }

        // LinearExpr - LinearExpr
        impl Sub for LinearExpr<$var_type> {
            type Output = Self;

            fn sub(mut self, other: Self) -> Self {
                for (var, coeff) in other.terms {
                    *self.terms.entry(var).or_insert(0.0) -= coeff;
                }
                self.constant -= other.constant;
                self
            }
        }

        // -LinearExpr
        impl Neg for LinearExpr<$var_type> {
            type Output = Self;

            fn neg(mut self) -> Self {
                for coeff in self.terms.values_mut() {
                    *coeff = -*coeff;
                }
                self.constant = -self.constant;
                self
            }
        }

        $(
            // From<numeric> for LinearExpr
            impl From<$num_type> for LinearExpr<$var_type> {
                fn from(constant: $num_type) -> Self {
                    LinearExpr::with_constant(constant as f64)
                }
            }

            // ExprVariable + numeric
            impl Add<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0);
                    LinearExpr::with_terms_and_constant(terms, constant as f64)
                }
            }

            // numeric + ExprVariable
            impl Add<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, var: $var_type) -> LinearExpr<$var_type> {
                    var + self
                }
            }

            // ExprVariable - numeric
            impl Sub<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0);
                    LinearExpr::with_terms_and_constant(terms, -(constant as f64))
                }
            }

            // numeric - ExprVariable
            impl Sub<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, var: $var_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(var, -1.0);
                    LinearExpr::with_terms_and_constant(terms, self as f64)
                }
            }

            // ExprVariable * numeric
            impl Mul<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, constant as f64);
                    LinearExpr::with_terms(terms)
                }
            }

            // numeric * ExprVariable
            impl Mul<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, var: $var_type) -> LinearExpr<$var_type> {
                    var * self
                }
            }

            // ExprVariable / numeric
            impl Div<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn div(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0 / (constant as f64));
                    LinearExpr::with_terms(terms)
                }
            }

            // numeric / ExprVariable
            impl Div<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn div(self, var: $var_type) -> LinearExpr<$var_type> {
                    var / self
                }
            }

            // LinearExpr + numeric
            impl Add<$num_type> for LinearExpr<$var_type> {
                type Output = Self;

                fn add(mut self, constant: $num_type) -> Self {
                    self.constant += constant as f64;
                    self
                }
            }

            // numeric + LinearExpr
            impl Add<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, mut expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    expr.constant += self as f64;
                    expr
                }
            }

            // LinearExpr - numeric
            impl Sub<$num_type> for LinearExpr<$var_type> {
                type Output = Self;

                fn sub(mut self, constant: $num_type) -> Self {
                    self.constant -= constant as f64;
                    self
                }
            }

            // numeric - LinearExpr
            impl Sub<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    -expr + self
                }
            }

            // LinearExpr * numeric
            impl Mul<$num_type> for LinearExpr<$var_type> {
                type Output = Self;

                fn mul(mut self, constant: $num_type) -> Self {
                    for coeff in self.terms.values_mut() {
                        *coeff *= constant as f64;
                    }
                    self.constant *= constant as f64;
                    self
                }
            }

            // numeric * LinearExpr
            impl Mul<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    expr * self
                }
            }

            // LinearExpr / numeric
            impl Div<$num_type> for LinearExpr<$var_type> {
                type Output = Self;

                fn div(self, constant: $num_type) -> Self {
                    self * (1.0 / constant as f64)
                }
            }

            // numeric / LinearExpr
            impl Div<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn div(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    expr / self
                }
            }
        )*
    };
}

pub(crate) use impl_expr_ops;
