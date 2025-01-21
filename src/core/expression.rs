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

macro_rules! impl_expr_display {
    ($var_type:ty) => {
        impl fmt::Display for LinearExpr<$var_type> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut first = true;

                for (var, &coefficient) in &self.terms {
                    // Skip zero coefficients
                    if coefficient == 0.0 {
                        continue;
                    }

                    // Print the sign if needed (based on first or not)
                    if !first {
                        if coefficient > 0.0 {
                            write!(f, " + ")?;
                        } else {
                            write!(f, " - ")?;
                        }
                    }

                    // Formatting the coefficient (with limited precision for readability)
                    let coefficient_str = match coefficient {
                        1.0 => String::new(),
                        -1.0 => {
                            if first {
                                String::from("-")
                            } else {
                                String::new()
                            }
                        }
                        _ => format!(
                            "{:.2} *",
                            if first {
                                coefficient
                            } else {
                                coefficient.abs()
                            }
                        ), // Limit to 2 decimal places
                    };

                    // If the coefficient is not 0 or 1 or -1, print the coefficient followed by a space and the variable
                    if coefficient != 1.0 && coefficient != -1.0 {
                        write!(f, "{} ", coefficient_str)?;
                    } else {
                        write!(f, "{}", coefficient_str)?; // No space if it's just '1' or '-1'
                    }

                    // Print the variable
                    write!(f, "{}", var)?;

                    first = false;
                }

                // Handle constant term
                if self.constant != 0.0 || first {
                    if !first {
                        if self.constant > 0.0 {
                            write!(f, " + ")?;
                        } else {
                            write!(f, " - ")?;
                        }
                    }
                    write!(
                        f,
                        "{:.2}",
                        if first {
                            self.constant
                        } else {
                            self.constant.abs()
                        }
                    )?;
                }

                Ok(())
            }
        }
    };
}

// ------------------- LinearExpr ops --------------------
// 1. LinearExpr + LinearExpr
// 2. LinearExpr - LinearExpr
// 3. -LinearExpr
// ------------------ ExprVariable ops -------------------
// 4. From<ExprVariable> for LinearExpr
// 5. -ExprVariable
// 6. ExprVariable + ExprVariable
// 7. ExprVariable - ExprVariable
// -------- Mixed ExprVariable and LinearExpr ops --------
// 8. ExprVariable + LinearExpr
// 9. ExprVariable - LinearExpr
// 10. LinearExpr + ExprVariable
// 11. LinearExpr - ExprVariable
// -------------- Numeric on LinearExpr ops --------------
// 12. From<numeric> for LinearExpr
// 13. LinearExpr + numeric
// 14. numeric + LinearExpr
// 15. LinearExpr - numeric
// 16. numeric - LinearExpr
// 17. LinearExpr * numeric
// 18. numeric * LinearExpr
// 19. LinearExpr / numeric
// 20. numeric / LinearExpr
// ------------- Numeric on ExprVariable ops -------------
// 21. ExprVariable + numeric
// 22. numeric + ExprVariable
// 23. ExprVariable - numeric
// 24. numeric - ExprVariable
// 25. ExprVariable * numeric
// 26. numeric * ExprVariable
// 27. ExprVariable / numeric
// 28. numeric / ExprVariable

macro_rules! impl_expr_ops {
    ($var_type:ty, [$($num_type:ty),* $(,)?]) => {
        use std::collections::HashMap;
        use std::ops::{Add, Div, Mul, Neg, Sub};
        use crate::core::expression::LinearExpr;

        // 1. LinearExpr + LinearExpr
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

        // 2. LinearExpr - LinearExpr
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

        // 3. -LinearExpr
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

        // 4. Implement From<ExprVariable> for LinearExpr
        impl From<$var_type> for LinearExpr<$var_type> {
            fn from(var: $var_type) -> Self {
                let mut terms = HashMap::new();
                terms.insert(var, 1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // 5. -ExprVariable
        impl Neg for $var_type {
            type Output = LinearExpr<$var_type>;

            fn neg(self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self, -1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // 6. ExprVariable + ExprVariable
        impl Add for $var_type {
            type Output = LinearExpr<$var_type>;

            fn add(self, other: Self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self, 1.0);
                terms.insert(other, 1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // 7. ExprVariable - ExprVariable
        impl Sub for $var_type {
            type Output = LinearExpr<$var_type>;

            fn sub(self, other: Self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self, 1.0);
                terms.insert(other, -1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // 8. ExprVariable + LinearExpr
        impl Add<LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;

            fn add(self, mut expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                expr.add_term(self, 1.0);
                expr
            }
        }

        // 9. ExprVariable - LinearExpr
        impl Sub<LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;

            fn sub(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                -expr + self
            }
        }

        // 10. LinearExpr + ExprVariable
        impl Add<$var_type> for LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn add(mut self, var: $var_type) -> LinearExpr<$var_type> {
                self.add_term(var, 1.0);
                self
            }
        }

        // 11. LinearExpr - ExprVariable
        impl Sub<$var_type> for LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn sub(mut self, var: $var_type) -> LinearExpr<$var_type> {
                self.add_term(var, -1.0);
                self
            }
        }

        $(
            // 12. From<numeric> for LinearExpr
            impl From<$num_type> for LinearExpr<$var_type> {
                fn from(constant: $num_type) -> Self {
                    LinearExpr::with_constant(constant as f64)
                }
            }

            // 13. LinearExpr + numeric
            impl Add<$num_type> for LinearExpr<$var_type> {
            type Output = Self;

                fn add(mut self, constant: $num_type) -> Self {
                    self.constant += constant as f64;
                    self
                }
            }

            // 14. numeric + LinearExpr
            impl Add<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, mut expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    expr.constant += self as f64;
                    expr
            }
        }

            // 15. LinearExpr - numeric
            impl Sub<$num_type> for LinearExpr<$var_type> {
            type Output = Self;

                fn sub(mut self, constant: $num_type) -> Self {
                    self.constant -= constant as f64;
                    self
                }
            }

            // 16. numeric - LinearExpr
            impl Sub<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    -expr + self
            }
        }

            // 17. LinearExpr * numeric
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

            // 18. numeric * LinearExpr
            impl Mul<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    expr * self
                }
            }

            // 19. LinearExpr / numeric
            impl Div<$num_type> for LinearExpr<$var_type> {
                type Output = Self;

                fn div(self, constant: $num_type) -> Self {
                    self * (1.0 / constant as f64)
                }
            }

            // 20. numeric / LinearExpr
            impl Div<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn div(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    expr / self
                }
            }

            // 21. ExprVariable + numeric
            impl Add<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0);
                    LinearExpr::with_terms_and_constant(terms, constant as f64)
                }
            }

            // 22. numeric + ExprVariable
            impl Add<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, var: $var_type) -> LinearExpr<$var_type> {
                    var + self
                }
            }

            // 23. ExprVariable - numeric
            impl Sub<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0);
                    LinearExpr::with_terms_and_constant(terms, -(constant as f64))
                }
            }

            // 24. numeric - ExprVariable
            impl Sub<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, var: $var_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(var, -1.0);
                    LinearExpr::with_terms_and_constant(terms, self as f64)
                }
            }

            // 25. ExprVariable * numeric
            impl Mul<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, constant as f64);
                    LinearExpr::with_terms(terms)
                }
            }

            // 26. numeric * ExprVariable
            impl Mul<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, var: $var_type) -> LinearExpr<$var_type> {
                    var * self
                }
            }

            // 27. ExprVariable / numeric
            impl Div<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn div(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0 / (constant as f64));
                    LinearExpr::with_terms(terms)
                }
            }

            // 28. numeric / ExprVariable
            impl Div<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn div(self, var: $var_type) -> LinearExpr<$var_type> {
                    var / self
                }
            }
        )*
    };
}

pub(crate) use impl_expr_display;
pub(crate) use impl_expr_ops;
