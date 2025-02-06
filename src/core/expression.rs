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

    pub fn with_term(var: T, coefficient: f64) -> Self {
        let mut terms = HashMap::new();
        terms.insert(var, coefficient);
        Self {
            terms,
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

    pub fn remove_term(&mut self, var: &T) -> Option<f64> {
        self.terms.remove(var)
    }

    pub fn add_expr(&mut self, other: &Self) {
        for (var, &coefficient) in &other.terms {
            self.add_term(var.clone(), coefficient);
        }
        self.constant += other.constant;
    }

    pub fn sub_expr(&mut self, other: &Self) {
        for (var, coefficient) in &other.terms {
            self.add_term(var.clone(), -coefficient);
        }
        self.constant -= other.constant;
    }

    pub fn add_constant(&mut self, constant: f64) {
        self.constant += constant;
    }

    pub fn replace_var_with_expr(
        &mut self,
        var: T,
        replacement_expr: &LinearExpr<T>,
    ) -> Option<f64> {
        if let Some(coefficient) = self.remove_term(&var) {
            let mut replacement_scaled = replacement_expr.clone();
            replacement_scaled.scale(coefficient);

            self.add_expr(&replacement_scaled);
            Some(coefficient)
        } else {
            None
        }
    }

    pub fn scale(&mut self, scalar: f64) {
        for coefficient in self.terms.values_mut() {
            *coefficient *= scalar;
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
// ------------- Numeric on ExprVariable ops -------------
// 20. ExprVariable + numeric
// 21. numeric + ExprVariable
// 22. ExprVariable - numeric
// 23. numeric - ExprVariable
// 24. ExprVariable * numeric
// 25. numeric * ExprVariable
// 26. ExprVariable / numeric

macro_rules! impl_expr_ops {
    ($var_type:ty, [$($num_type:ty),* $(,)?]) => {
        use std::collections::HashMap;
        use std::ops::{Add, Div, Mul, Neg, Sub};
        use crate::core::expression::LinearExpr;

        // ==================== LinearExpr Operations ====================

        // 1. LinearExpr + LinearExpr (owned and reference combinations)
        impl Add for LinearExpr<$var_type> {
            type Output = Self;

            fn add(self, other: Self) -> Self {
                let mut result = self.clone();
                for (var, coefficient) in other.terms {
                    *result.terms.entry(var).or_insert(0.0) += coefficient;
                }
                result.constant += other.constant;
                result
            }
        }

        impl<'a> Add for &'a LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn add(self, other: Self) -> LinearExpr<$var_type> {
                let mut result = self.clone();
                for (var, coefficient) in &other.terms {
                    *result.terms.entry(var.clone()).or_insert(0.0) += coefficient;
                }
                result.constant += other.constant;
                result
            }
        }

        impl<'a> Add<&'a LinearExpr<$var_type>> for LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn add(self, other: &'a LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                let mut result = self.clone();
                for (var, coefficient) in &other.terms {
                    *result.terms.entry(var.clone()).or_insert(0.0) += coefficient;
                }
                result.constant += other.constant;
                result
            }
        }

        // 2. LinearExpr - LinearExpr (owned and reference combinations)
        impl Sub for LinearExpr<$var_type> {
            type Output = Self;

            fn sub(self, other: Self) -> Self {
                let mut result = self.clone();
                for (var, coefficient) in other.terms {
                    *result.terms.entry(var).or_insert(0.0) -= coefficient;
                }
                result.constant -= other.constant;
                result
            }
        }

        impl<'a> Sub for &'a LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn sub(self, other: Self) -> LinearExpr<$var_type> {
                let mut result = self.clone();
                for (var, coefficient) in &other.terms {
                    *result.terms.entry(var.clone()).or_insert(0.0) -= coefficient;
                }
                result.constant -= other.constant;
                result
            }
        }

        impl<'a> Sub<&'a LinearExpr<$var_type>> for LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn sub(self, other: &'a LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                let mut result = self.clone();
                for (var, coefficient) in &other.terms {
                    *result.terms.entry(var.clone()).or_insert(0.0) -= coefficient;
                }
                result.constant -= other.constant;
                result
            }
        }

        // 3. -LinearExpr (owned and reference)
        impl Neg for LinearExpr<$var_type> {
            type Output = Self;

            fn neg(self) -> Self {
                let mut result = self.clone();
                for coefficient in result.terms.values_mut() {
                    *coefficient = -*coefficient;
                }
                result.constant = -result.constant;
                result
            }
        }

        impl<'a> Neg for &'a LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn neg(self) -> LinearExpr<$var_type> {
                let mut result = self.clone();
                for coefficient in result.terms.values_mut() {
                    *coefficient = -*coefficient;
                }
                result.constant = -result.constant;
                result
            }
        }

        // ==================== ExprVariable Operations ====================

        // 4. From<ExprVariable> for LinearExpr (owned and reference)
        impl From<$var_type> for LinearExpr<$var_type> {
            fn from(var: $var_type) -> Self {
                let mut terms = HashMap::new();
                terms.insert(var, 1.0);
                LinearExpr::with_terms(terms)
            }
        }

        impl<'a> From<&'a $var_type> for LinearExpr<$var_type> {
            fn from(var: &'a $var_type) -> Self {
                let mut terms = HashMap::new();
                terms.insert(var.clone(), 1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // 5. -ExprVariable (owned and reference)
        impl Neg for $var_type {
            type Output = LinearExpr<$var_type>;

            fn neg(self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self, -1.0);
                LinearExpr::with_terms(terms)
            }
        }

        impl<'a> Neg for &'a $var_type {
            type Output = LinearExpr<$var_type>;

            fn neg(self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self.clone(), -1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // 6. ExprVariable + ExprVariable (owned and reference combinations)
        impl Add for $var_type {
            type Output = LinearExpr<$var_type>;

            fn add(self, other: Self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self, 1.0);
                terms.insert(other, 1.0);
                LinearExpr::with_terms(terms)
            }
        }

        impl<'a> Add for &'a $var_type {
            type Output = LinearExpr<$var_type>;

            fn add(self, other: Self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self.clone(), 1.0);
                terms.insert(other.clone(), 1.0);
                LinearExpr::with_terms(terms)
            }
        }

        impl<'a> Add<&'a $var_type> for $var_type {
            type Output = LinearExpr<$var_type>;

            fn add(self, other: &'a Self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self, 1.0);
                terms.insert(other.clone(), 1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // 7. ExprVariable - ExprVariable (owned and reference combinations)
        impl Sub for $var_type {
            type Output = LinearExpr<$var_type>;

            fn sub(self, other: Self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self, 1.0);
                terms.insert(other, -1.0);
                LinearExpr::with_terms(terms)
            }
        }

        impl<'a> Sub for &'a $var_type {
            type Output = LinearExpr<$var_type>;

            fn sub(self, other: Self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self.clone(), 1.0);
                terms.insert(other.clone(), -1.0);
                LinearExpr::with_terms(terms)
            }
        }

        impl<'a> Sub<&'a $var_type> for $var_type {
            type Output = LinearExpr<$var_type>;

            fn sub(self, other: &'a Self) -> LinearExpr<$var_type> {
                let mut terms = HashMap::new();
                terms.insert(self, 1.0);
                terms.insert(other.clone(), -1.0);
                LinearExpr::with_terms(terms)
            }
        }

        // ==================== Mixed Operations ====================

        // 8. ExprVariable + LinearExpr (owned and reference combinations)
        impl Add<LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;

            fn add(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                let mut result = expr.clone();
                result.add_term(self, 1.0);
                result
            }
        }

        impl<'a> Add<&'a LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;

            fn add(self, expr: &'a LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                let mut result = expr.clone();
                result.add_term(self, 1.0);
                result
            }
        }

        impl<'a> Add<LinearExpr<$var_type>> for &'a $var_type {
            type Output = LinearExpr<$var_type>;

            fn add(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                let mut result = expr.clone();
                result.add_term(self.clone(), 1.0);
                result
            }
        }

        // 9. ExprVariable - LinearExpr (owned and reference combinations)
        impl Sub<LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;

            fn sub(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                let mut result = -expr;
                result.add_term(self, 1.0);
                result
            }
        }

        impl<'a> Sub<&'a LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;

            fn sub(self, expr: &'a LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                let mut result = -expr.clone();
                result.add_term(self, 1.0);
                result
            }
        }

        impl<'a> Sub<LinearExpr<$var_type>> for &'a $var_type {
            type Output = LinearExpr<$var_type>;

            fn sub(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                let mut result = -expr;
                result.add_term(self.clone(), 1.0);
                result
            }
        }

        // 10. LinearExpr + ExprVariable (owned and reference combinations)
        impl Add<$var_type> for LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn add(self, var: $var_type) -> LinearExpr<$var_type> {
                let mut result = self.clone();
                result.add_term(var, 1.0);
                result
            }
        }

        impl<'a> Add<&'a $var_type> for LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn add(self, var: &'a $var_type) -> LinearExpr<$var_type> {
                let mut result = self.clone();
                result.add_term(var.clone(), 1.0);
                result
            }
        }

        impl<'a> Add<$var_type> for &LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn add(self, var: $var_type) -> LinearExpr<$var_type> {
                let mut result = self.clone();
                result.add_term(var, 1.0);
                result
            }
        }

        // 11. LinearExpr - ExprVariable (owned and reference combinations)
        impl Sub<$var_type> for LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn sub(self, var: $var_type) -> LinearExpr<$var_type> {
                let mut result = self.clone();
                result.add_term(var, -1.0);
                result
            }
        }

        impl<'a> Sub<&'a $var_type> for LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn sub(self, var: &'a $var_type) -> LinearExpr<$var_type> {
                let mut result = self.clone();
                result.add_term(var.clone(), -1.0);
                result
            }
        }

        impl<'a> Sub<$var_type> for &LinearExpr<$var_type> {
            type Output = LinearExpr<$var_type>;

            fn sub(self, var: $var_type) -> LinearExpr<$var_type> {
                let mut result = self.clone();
                result.add_term(var, -1.0);
                result
            }
        }

        // ==================== Numeric Operations ====================
        $(
            // 12. From<numeric> for LinearExpr
            impl From<$num_type> for LinearExpr<$var_type> {
                fn from(constant: $num_type) -> Self {
                    LinearExpr::with_constant(constant as f64)
                }
            }

            // 13. LinearExpr + numeric (owned and reference)
            impl Add<$num_type> for LinearExpr<$var_type> {
                type Output = Self;

                fn add(self, constant: $num_type) -> Self {
                    let mut result = self.clone();
                    result.constant += constant as f64;
                    result
                }
            }

            impl<'a> Add<$num_type> for &'a LinearExpr<$var_type> {
                type Output = LinearExpr<$var_type>;

                fn add(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut result = self.clone();
                    result.constant += constant as f64;
                    result
                }
            }

            // 14. numeric + LinearExpr (owned and reference)
            impl Add<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    let mut result = expr.clone();
                    result.constant += self as f64;
                    result
                }
            }

            impl<'a> Add<&'a LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, expr: &'a LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    let mut result = expr.clone();
                    result.constant += self as f64;
                    result
                }
            }

            // 15. LinearExpr - numeric (owned and reference)
            impl Sub<$num_type> for LinearExpr<$var_type> {
                type Output = Self;

                fn sub(self, constant: $num_type) -> Self {
                    let mut result = self.clone();
                    result.constant -= constant as f64;
                    result
                }
            }

            impl<'a> Sub<$num_type> for &'a LinearExpr<$var_type> {
                type Output = LinearExpr<$var_type>;

                fn sub(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut result = self.clone();
                    result.constant -= constant as f64;
                    result
                }
            }

            // 16. numeric - LinearExpr (owned and reference)
            impl Sub<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    let mut result = -expr;
                    result.constant += self as f64;
                    result
                }
            }

            impl<'a> Sub<&'a LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, expr: &'a LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    let mut result = -expr.clone();
                    result.constant += self as f64;
                    result
                }
            }

           // 17. LinearExpr * numeric (owned and reference)
            impl Mul<$num_type> for LinearExpr<$var_type> {
                type Output = Self;

                fn mul(self, constant: $num_type) -> Self {
                    let mut result = self.clone();
                    for coefficient in result.terms.values_mut() {
                        *coefficient *= constant as f64;
                    }
                    result.constant *= constant as f64;
                    result
                }
            }

            impl<'a> Mul<$num_type> for &'a LinearExpr<$var_type> {
                type Output = LinearExpr<$var_type>;

                fn mul(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut result = self.clone();
                    for coefficient in result.terms.values_mut() {
                        *coefficient *= constant as f64;
                    }
                    result.constant *= constant as f64;
                    result
                }
            }

            // 18. numeric * LinearExpr (owned and reference)
            impl Mul<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    expr * self
                }
            }

            impl<'a> Mul<&'a LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, expr: &'a LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    expr * self
                }
            }

            // 19. LinearExpr / numeric (owned and reference)
            impl Div<$num_type> for LinearExpr<$var_type> {
                type Output = Self;

                fn div(self, constant: $num_type) -> Self {
                    let mut result = self.clone();
                    let divisor = constant as f64;
                    for coefficient in result.terms.values_mut() {
                        *coefficient /= divisor;
                    }
                    result.constant /= divisor;
                    result
                }
            }

            impl<'a> Div<$num_type> for &'a LinearExpr<$var_type> {
                type Output = LinearExpr<$var_type>;

                fn div(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut result = self.clone();
                    let divisor = constant as f64;
                    for coefficient in result.terms.values_mut() {
                        *coefficient /= divisor;
                    }
                    result.constant /= divisor;
                    result
                }
            }

            // 20. ExprVariable + numeric (owned and reference)
            impl Add<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0);
                    LinearExpr::with_terms_and_constant(terms, constant as f64)
                }
            }

            impl<'a> Add<$num_type> for &'a $var_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self.clone(), 1.0);
                    LinearExpr::with_terms_and_constant(terms, constant as f64)
                }
            }

            // 21. numeric + ExprVariable (owned and reference)
            impl Add<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, var: $var_type) -> LinearExpr<$var_type> {
                    var + self
                }
            }

            impl<'a> Add<&'a $var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn add(self, var: &'a $var_type) -> LinearExpr<$var_type> {
                    var + self
                }
            }

            // 22. ExprVariable - numeric (owned and reference)
            impl Sub<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0);
                    LinearExpr::with_terms_and_constant(terms, -(constant as f64))
                }
            }

            impl<'a> Sub<$num_type> for &'a $var_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self.clone(), 1.0);
                    LinearExpr::with_terms_and_constant(terms, -(constant as f64))
                }
            }

            // 23. numeric - ExprVariable (owned and reference)
            impl Sub<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, var: $var_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(var, -1.0);
                    LinearExpr::with_terms_and_constant(terms, self as f64)
                }
            }

            impl<'a> Sub<&'a $var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn sub(self, var: &'a $var_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(var.clone(), -1.0);
                    LinearExpr::with_terms_and_constant(terms, self as f64)
                }
            }

            // 24. ExprVariable * numeric (owned and reference)
            impl Mul<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, constant as f64);
                    LinearExpr::with_terms(terms)
                }
            }

            impl<'a> Mul<$num_type> for &'a $var_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self.clone(), constant as f64);
                    LinearExpr::with_terms(terms)
                }
            }

            // 25. numeric * ExprVariable (owned and reference)
            impl Mul<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, var: $var_type) -> LinearExpr<$var_type> {
                    var * self
                }
            }

            impl<'a> Mul<&'a $var_type> for $num_type {
                type Output = LinearExpr<$var_type>;

                fn mul(self, var: &'a $var_type) -> LinearExpr<$var_type> {
                    var * self
                }
            }

            // 26. ExprVariable / numeric (owned and reference)
            impl Div<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;

                fn div(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self, 1.0 / (constant as f64));
                    LinearExpr::with_terms(terms)
                }
            }

            impl<'a> Div<$num_type> for &'a $var_type {
                type Output = LinearExpr<$var_type>;

                fn div(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = HashMap::new();
                    terms.insert(self.clone(), 1.0 / (constant as f64));
                    LinearExpr::with_terms(terms)
                }
            }
        )*
    };
}

pub(crate) use impl_expr_display;
pub(crate) use impl_expr_ops;
