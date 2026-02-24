use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

/// A linear expression stored as a sorted sparse vector.
/// Invariants:
/// 1. `terms` is always sorted by Variable T.
/// 2. `terms` never contains coefficients with abs() < tolerance (effectively zero).
#[derive(Debug, Clone)]
pub struct LinearExpr<T: ExprVariable> {
    pub terms: Vec<(T, f64)>,
    pub constant: f64,
}

/// Trait for types that can be used as variables in a linear expression.
pub trait ExprVariable: Clone + Copy + Eq + Ord + fmt::Display {}

impl<T: ExprVariable> LinearExpr<T> {
    const TOLERANCE: f64 = 1e-10;

    pub fn new() -> Self {
        Self {
            terms: Vec::new(),
            constant: 0.0,
        }
    }

    pub fn with_term(var: T, coefficient: f64) -> Self {
        if coefficient.abs() < Self::TOLERANCE {
            return Self::new();
        }
        Self {
            terms: vec![(var, coefficient)],
            constant: 0.0,
        }
    }

    pub fn with_terms(mut terms: Vec<(T, f64)>) -> Self {
        // 1. Sort by variable to enable O(N) merging later
        terms.sort_by(|a, b| a.0.cmp(&b.0));

        // 2. Deduplicate (merge coefficients for same variable) and Filter Zeros
        let mut dedup_terms = Vec::with_capacity(terms.len());
        if !terms.is_empty() {
            let mut current_var = terms[0].0.clone();
            let mut current_coeff = terms[0].1;

            for (var, coeff) in terms.into_iter().skip(1) {
                if var == current_var {
                    current_coeff += coeff;
                } else {
                    if current_coeff.abs() >= Self::TOLERANCE {
                        dedup_terms.push((current_var, current_coeff));
                    }
                    current_var = var;
                    current_coeff = coeff;
                }
            }
            // Push the last one
            if current_coeff.abs() >= Self::TOLERANCE {
                dedup_terms.push((current_var, current_coeff));
            }
        }

        Self {
            terms: dedup_terms,
            constant: 0.0,
        }
    }

    pub fn with_constant(constant: f64) -> Self {
        Self {
            terms: Vec::new(),
            constant,
        }
    }

    pub fn with_terms_and_constant(terms: Vec<(T, f64)>, constant: f64) -> Self {
        let mut expr = Self::with_terms(terms);
        expr.constant = constant;
        expr
    }

    pub fn coefficient(&self, var: &T) -> f64 {
        self.terms
            .binary_search_by(|(v, _)| v.cmp(var))
            .map(|idx| self.terms[idx].1)
            .unwrap_or(0.0)
    }

    pub fn add_term(&mut self, var: T, coefficient: f64) {
        if coefficient.abs() < Self::TOLERANCE {
            return;
        }

        match self.terms.binary_search_by(|(v, _)| v.cmp(&var)) {
            Ok(idx) => {
                self.terms[idx].1 += coefficient;
                // Check if it became zero after addition
                if self.terms[idx].1.abs() < Self::TOLERANCE {
                    self.terms.remove(idx);
                }
            }
            Err(idx) => {
                self.terms.insert(idx, (var, coefficient));
            }
        }
    }

    pub fn remove_term(&mut self, var: &T) -> Option<f64> {
        if let Ok(idx) = self.terms.binary_search_by(|(v, _)| v.cmp(var)) {
            Some(self.terms.remove(idx).1)
        } else {
            None
        }
    }

    pub fn add_expr(&mut self, other: &Self) {
        self.add_scaled_expr(other, 1.0);
    }

    pub fn sub_expr(&mut self, other: &Self) {
        self.add_scaled_expr(other, -1.0);
    }

    pub fn add_scaled_expr(&mut self, other: &Self, scale: f64) {
        if other.terms.is_empty() {
            self.constant += other.constant * scale;
            return;
        }

        // We rebuild the vector. This is often faster than inserting into the middle
        // repeatedly (which is O(N^2)) when `other` has many terms.
        let mut new_terms = Vec::with_capacity(self.terms.len() + other.terms.len());
        let mut i = 0;
        let mut j = 0;

        while i < self.terms.len() && j < other.terms.len() {
            let (var_self, coeff_self) = &self.terms[i];
            let (var_other, coeff_other) = &other.terms[j];

            match var_self.cmp(var_other) {
                Ordering::Less => {
                    new_terms.push((var_self.clone(), *coeff_self));
                    i += 1;
                }
                Ordering::Greater => {
                    let scaled_val = coeff_other * scale;
                    if scaled_val.abs() > Self::TOLERANCE {
                        new_terms.push((var_other.clone(), scaled_val));
                    }
                    j += 1;
                }
                Ordering::Equal => {
                    let new_coeff = *coeff_self + (coeff_other * scale);
                    if new_coeff.abs() > Self::TOLERANCE {
                        new_terms.push((var_self.clone(), new_coeff));
                    }
                    i += 1;
                    j += 1;
                }
            }
        }

        // Append remaining from self
        if i < self.terms.len() {
            new_terms.extend_from_slice(&self.terms[i..]);
        }

        // Append remaining from other
        while j < other.terms.len() {
            let (var, coeff) = &other.terms[j];
            let scaled_val = coeff * scale;
            if scaled_val.abs() > Self::TOLERANCE {
                new_terms.push((var.clone(), scaled_val));
            }
            j += 1;
        }

        self.terms = new_terms;
        self.constant += other.constant * scale;
    }

    pub fn add_constant(&mut self, constant: f64) {
        self.constant += constant;
    }

    pub fn scale(&mut self, scalar: f64) {
        if scalar.abs() < Self::TOLERANCE {
            self.terms.clear();
            self.constant = 0.0;
            return;
        }

        // We might create zeros if the scalar is very small, so we must filter.
        self.terms.retain_mut(|(_, c)| {
            *c *= scalar;
            c.abs() > Self::TOLERANCE
        });
        self.constant *= scalar;
    }

    pub fn replace_var_with_expr(
        &mut self,
        var: T,
        replacement_expr: &LinearExpr<T>,
    ) -> Option<f64> {
        // 1. Remove the term (O(log N) + O(N) shift)
        if let Some(coefficient) = self.remove_term(&var) {
            // 2. Merge the new expression (O(N + M))
            // This replaces the old O(M * N) loop.
            self.add_scaled_expr(replacement_expr, coefficient);
            Some(coefficient)
        } else {
            None
        }
    }
}

// ============================================================
//  GENERIC OPERATOR IMPLEMENTATIONS
// ============================================================

// --- Conversions ---

/// Implements `From<f64>` for `LinearExpr`.
impl<T: ExprVariable> From<f64> for LinearExpr<T> {
    fn from(constant: f64) -> Self {
        LinearExpr::with_constant(constant)
    }
}

// --- Negation (-Expr) ---

/// Implements `-Expr`
impl<T: ExprVariable> Neg for LinearExpr<T> {
    type Output = Self;
    fn neg(mut self) -> Self {
        self.scale(-1.0);
        self
    }
}

/// Implements `-&Expr` (creates new Owned)
impl<'a, T: ExprVariable> Neg for &'a LinearExpr<T> {
    type Output = LinearExpr<T>;
    fn neg(self) -> LinearExpr<T> {
        let mut new_expr = self.clone();
        new_expr.scale(-1.0);
        new_expr
    }
}

// --- Addition (Expr + X) ---

/// Implements `Expr + Expr` (Reuse LHS)
impl<T: ExprVariable> Add<LinearExpr<T>> for LinearExpr<T> {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self.add_expr(&rhs);
        self
    }
}

/// Implements `Expr + &Expr` (Reuse LHS)
impl<'a, T: ExprVariable> Add<&'a LinearExpr<T>> for LinearExpr<T> {
    type Output = Self;
    fn add(mut self, rhs: &'a LinearExpr<T>) -> Self {
        self.add_expr(rhs);
        self
    }
}

/// Implements `Expr + Var`
impl<T: ExprVariable> Add<T> for LinearExpr<T> {
    type Output = Self;
    fn add(mut self, var: T) -> Self {
        self.add_term(var, 1.0);
        self
    }
}

/// Implements `Expr + f64`
impl<T: ExprVariable> Add<f64> for LinearExpr<T> {
    type Output = Self;
    fn add(mut self, constant: f64) -> Self {
        self.add_constant(constant);
        self
    }
}

/// Implements `&Expr + f64` (creates new Owned)
impl<'a, T: ExprVariable> Add<f64> for &'a LinearExpr<T> {
    type Output = LinearExpr<T>;
    fn add(self, constant: f64) -> LinearExpr<T> {
        let mut new_expr = self.clone();
        new_expr.add_constant(constant);
        new_expr
    }
}

/// Implements `Expr += Expr`
impl<T: ExprVariable> AddAssign for LinearExpr<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.add_expr(&rhs);
    }
}

// --- Subtraction (Expr - X) ---

/// Implements `Expr - Expr` (Reuse LHS)
impl<T: ExprVariable> Sub<LinearExpr<T>> for LinearExpr<T> {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self {
        self.sub_expr(&rhs);
        self
    }
}

/// Implements `Expr - &Expr` (Reuse LHS)
impl<'a, T: ExprVariable> Sub<&'a LinearExpr<T>> for LinearExpr<T> {
    type Output = Self;
    fn sub(mut self, rhs: &'a LinearExpr<T>) -> Self {
        self.sub_expr(rhs);
        self
    }
}

/// Implements `Expr - Var`
impl<T: ExprVariable> Sub<T> for LinearExpr<T> {
    type Output = Self;
    fn sub(mut self, var: T) -> Self {
        self.add_term(var, -1.0);
        self
    }
}

/// Implements `Expr - f64`
impl<T: ExprVariable> Sub<f64> for LinearExpr<T> {
    type Output = Self;
    fn sub(mut self, constant: f64) -> Self {
        self.add_constant(-constant);
        self
    }
}

/// Implements `&Expr - f64` (creates new Owned)
impl<'a, T: ExprVariable> Sub<f64> for &'a LinearExpr<T> {
    type Output = LinearExpr<T>;
    fn sub(self, constant: f64) -> LinearExpr<T> {
        let mut new_expr = self.clone();
        new_expr.add_constant(-constant);
        new_expr
    }
}

/// Implements `Expr -= Expr`
impl<T: ExprVariable> SubAssign for LinearExpr<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_expr(&rhs);
    }
}

// --- Multiplication (Expr * f64) ---

/// Implements `Expr * f64`
impl<T: ExprVariable> Mul<f64> for LinearExpr<T> {
    type Output = Self;
    fn mul(mut self, scalar: f64) -> Self {
        self.scale(scalar);
        self
    }
}

/// Implements `&Expr * f64` (creates new Owned)
impl<'a, T: ExprVariable> Mul<f64> for &'a LinearExpr<T> {
    type Output = LinearExpr<T>;
    fn mul(self, scalar: f64) -> LinearExpr<T> {
        let mut new_expr = self.clone();
        new_expr.scale(scalar);
        new_expr
    }
}

// --- Division (Expr / f64) ---

/// Implements `Expr / f64`
impl<T: ExprVariable> Div<f64> for LinearExpr<T> {
    type Output = Self;
    fn div(mut self, scalar: f64) -> Self {
        self.scale(1.0 / scalar);
        self
    }
}

/// Implements `&Expr / f64` (creates new Owned)
impl<'a, T: ExprVariable> Div<f64> for &'a LinearExpr<T> {
    type Output = LinearExpr<T>;
    fn div(self, scalar: f64) -> LinearExpr<T> {
        let mut new_expr = self.clone();
        new_expr.scale(1.0 / scalar);
        new_expr
    }
}

// ============================================================
//  MACRO: Boilerplate for "Orphan Rule" Cases
//  Handles interactions where `LinearExpr` is on the Right Hand Side.
// ============================================================

macro_rules! impl_expr_ops {
    ($var_type:ty) => {
        use crate::common::expression::LinearExpr;
        use std::ops::{Add, Div, Mul, Neg, Sub};

        // --- 1. Variable Conversions ---

        /// Implements `From<Var>` for `LinearExpr`
        impl From<$var_type> for LinearExpr<$var_type> {
            fn from(var: $var_type) -> Self {
                LinearExpr::with_term(var, 1.0)
            }
        }

        /// Implements `-Var`
        impl Neg for $var_type {
            type Output = LinearExpr<$var_type>;
            fn neg(self) -> Self::Output {
                LinearExpr::with_term(self, -1.0)
            }
        }

        // --- 2. Variable Arithmetic (Var op Var) ---

        /// Implements `Var + Var`
        impl Add<$var_type> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn add(self, other: Self) -> Self::Output {
                LinearExpr::with_terms(vec![(self, 1.0), (other, 1.0)])
            }
        }

        /// Implements `Var - Var`
        impl Sub<$var_type> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn sub(self, other: Self) -> Self::Output {
                LinearExpr::with_terms(vec![(self, 1.0), (other, -1.0)])
            }
        }

        // --- 3. Variable/Scalar -> Expression Interactions ---
        // (Where Expr is on the Right Hand Side)

        /// Implements `Var + Expr`
        impl Add<LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn add(self, mut expr: LinearExpr<$var_type>) -> Self::Output {
                expr.add_term(self, 1.0);
                expr
            }
        }

        /// Implements `Var - Expr`
        impl Sub<LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn sub(self, mut expr: LinearExpr<$var_type>) -> Self::Output {
                // var - expr => -expr + var
                expr.scale(-1.0);
                expr.add_term(self, 1.0);
                expr
            }
        }

        /// Implements `f64 + Expr`
        impl Add<LinearExpr<$var_type>> for f64 {
            type Output = LinearExpr<$var_type>;
            fn add(self, mut expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                expr.add_constant(self);
                expr
            }
        }

        /// Implements `f64 + &Expr`
        impl<'a> Add<&'a LinearExpr<$var_type>> for f64 {
            type Output = LinearExpr<$var_type>;
            fn add(self, expr: &'a LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                let mut new_expr = expr.clone();
                new_expr.add_constant(self);
                new_expr
            }
        }

        /// Implements `f64 - Expr`
        impl Sub<LinearExpr<$var_type>> for f64 {
            type Output = LinearExpr<$var_type>;
            fn sub(self, mut expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                // constant - expr => -expr + constant
                expr.scale(-1.0);
                expr.add_constant(self);
                expr
            }
        }

        /// Implements `f64 - &Expr`
        impl<'a> Sub<&'a LinearExpr<$var_type>> for f64 {
            type Output = LinearExpr<$var_type>;
            fn sub(self, expr: &'a LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                let mut new_expr = expr.clone();
                new_expr.scale(-1.0);
                new_expr.add_constant(self);
                new_expr
            }
        }

        /// Implements `f64 * Expr`
        impl Mul<LinearExpr<$var_type>> for f64 {
            type Output = LinearExpr<$var_type>;
            fn mul(self, mut expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                expr.scale(self);
                expr
            }
        }

        /// Implements `f64 * &Expr`
        impl<'a> Mul<&'a LinearExpr<$var_type>> for f64 {
            type Output = LinearExpr<$var_type>;
            fn mul(self, expr: &'a LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                let mut new_expr = expr.clone();
                new_expr.scale(self);
                new_expr
            }
        }

        // --- 4. Scalar Arithmetic (Var op f64) ---

        /// Implements `Var + f64`
        impl Add<f64> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn add(self, constant: f64) -> LinearExpr<$var_type> {
                LinearExpr::with_terms_and_constant(vec![(self, 1.0)], constant)
            }
        }

        /// Implements `f64 + Var`
        impl Add<$var_type> for f64 {
            type Output = LinearExpr<$var_type>;
            fn add(self, var: $var_type) -> LinearExpr<$var_type> {
                LinearExpr::with_terms_and_constant(vec![(var, 1.0)], self)
            }
        }

        /// Implements `Var - f64`
        impl Sub<f64> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn sub(self, constant: f64) -> LinearExpr<$var_type> {
                LinearExpr::with_terms_and_constant(vec![(self, 1.0)], -constant)
            }
        }

        /// Implements `f64 - Var`
        impl Sub<$var_type> for f64 {
            type Output = LinearExpr<$var_type>;
            fn sub(self, var: $var_type) -> LinearExpr<$var_type> {
                LinearExpr::with_terms_and_constant(vec![(var, -1.0)], self)
            }
        }

        /// Implements `Var * f64`
        impl Mul<f64> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn mul(self, constant: f64) -> LinearExpr<$var_type> {
                LinearExpr::with_term(self, constant)
            }
        }

        /// Implements `f64 * Var`
        impl Mul<$var_type> for f64 {
            type Output = LinearExpr<$var_type>;
            fn mul(self, var: $var_type) -> LinearExpr<$var_type> {
                LinearExpr::with_term(var, self)
            }
        }

        /// Implements `Var / f64`
        impl Div<f64> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn div(self, constant: f64) -> LinearExpr<$var_type> {
                LinearExpr::with_term(self, 1.0 / constant)
            }
        }
    };
}

macro_rules! impl_expr_display {
    ($var_type:ty) => {
        impl fmt::Display for LinearExpr<$var_type> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut first = true;

                for (var, coefficient) in &self.terms {
                    let coefficient = *coefficient;

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

pub(crate) use impl_expr_display;
pub(crate) use impl_expr_ops;
