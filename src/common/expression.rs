use std::cmp::Ordering;
use std::fmt;

/// A linear expression stored as a sorted sparse vector.
/// Invariants:
/// 1. `terms` is always sorted by Variable T.
/// 2. `terms` never contains coefficients with abs() < tolerance (effectively zero).
#[derive(Debug, Clone)]
pub struct LinearExpr<T: ExprVariable> {
    pub terms: Vec<(T, f64)>,
    pub constant: f64,
}

pub trait ExprVariable: Clone + Eq + Ord + fmt::Display {}

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

// ============================================================
//  CORE LOGIC: LinearExpr Operations
// ============================================================
// --- Add ---
// --- Sub ---
// --- Neg ---
// ============================================================
//  INTERACTION: ExprVariable <-> LinearExpr
// ============================================================
// --- Expr From Variable ---
// --- Neg Variable ---
// --- Var + Var ---
// --- Var - Var ---
// --- Var + Expr ---
// --- Expr + Var ---
// --- Var - Expr ---
// --- Expr - Var ---
// ============================================================
//  NUMERIC OPERATIONS
// ============================================================
// --- Expr From Numeric ---
// --- Expr + Num ---
// --- Num + Expr ---
// --- Expr - Num ---
// --- Num - Expr ---
// --- Expr * Num ---
// --- Num * Expr ---
// --- Expr / Num ---
// --- Var + Num ---
// --- Num + Var ---
// --- Var - Num ---
// --- Num - Var ---
// --- Var * Num ---
// --- Num * Var ---
// --- Var / Num ---
macro_rules! impl_expr_ops {
    ($var_type:ty, [$($num_type:ty),* $(,)?]) => {
        use std::ops::{Add, Div, Mul, Neg, Sub};
        use crate::common::expression::LinearExpr;

        // ============================================================
        //  HELPER MACROS: Automatic Reference Forwarding
        // ============================================================

        // Generates implementations for: &LHS + &RHS, &LHS + RHS, LHS + &RHS
        // by forwarding them to the value-based implementation: LHS + RHS
        macro_rules! forward_binop {
            (impl $trait:ident, $fn:ident for $lhs:ty, $rhs:ty) => {
                // &LHS op &RHS
                impl<'a, 'b> $trait<&'b $rhs> for &'a $lhs {
                    type Output = LinearExpr<$var_type>;
                    fn $fn(self, other: &'b $rhs) -> Self::Output {
                        self.clone().$fn(other.clone())
                    }
                }
                // &LHS op RHS
                impl<'a> $trait<$rhs> for &'a $lhs {
                    type Output = LinearExpr<$var_type>;
                    fn $fn(self, other: $rhs) -> Self::Output {
                        self.clone().$fn(other)
                    }
                }
                // LHS op &RHS
                impl<'a> $trait<&'a $rhs> for $lhs {
                    type Output = LinearExpr<$var_type>;
                    fn $fn(self, other: &'a $rhs) -> Self::Output {
                        self.$fn(other.clone())
                    }
                }
            };
        }

        // Generates implementations for: -&val
        // by forwarding to: -val
        macro_rules! forward_unop {
            (impl $trait:ident, $fn:ident for $target:ty) => {
                impl<'a> $trait for &'a $target {
                    type Output = LinearExpr<$var_type>;
                    fn $fn(self) -> Self::Output {
                        self.clone().$fn()
                    }
                }
            };
        }

        // ============================================================
        //  CORE LOGIC: LinearExpr Operations
        // ============================================================

        // --- Add ---
        impl Add<LinearExpr<$var_type>> for LinearExpr<$var_type> {
            type Output = Self;
            fn add(mut self, other: Self) -> Self {
                self.add_expr(&other);
                self
            }
        }
        forward_binop!(impl Add, add for LinearExpr<$var_type>, LinearExpr<$var_type>);

        // --- Sub ---
        impl Sub<LinearExpr<$var_type>> for LinearExpr<$var_type> {
            type Output = Self;
            fn sub(mut self, other: Self) -> Self {
                self.sub_expr(&other);
                self
            }
        }
        forward_binop!(impl Sub, sub for LinearExpr<$var_type>, LinearExpr<$var_type>);

        // --- Neg ---
        impl Neg for LinearExpr<$var_type> {
            type Output = Self;
            fn neg(mut self) -> Self {
                self.scale(-1.0);
                self
            }
        }
        forward_unop!(impl Neg, neg for LinearExpr<$var_type>);

        // ============================================================
        //  INTERACTION: ExprVariable <-> LinearExpr
        // ============================================================

        // --- Expr From Variable ---
        impl From<$var_type> for LinearExpr<$var_type> {
            fn from(var: $var_type) -> Self {
                LinearExpr::with_term(var, 1.0)
            }
        }
        
        // --- From &Variable ---
        impl<'a> From<&'a $var_type> for LinearExpr<$var_type> {
            fn from(var: &'a $var_type) -> Self {
                LinearExpr::with_term(var.clone(), 1.0)
            }
        }

        // --- Neg Variable ---
        impl Neg for $var_type {
            type Output = LinearExpr<$var_type>;
            fn neg(self) -> Self::Output {
                LinearExpr::with_term(self, -1.0)
            }
        }
        forward_unop!(impl Neg, neg for $var_type);

        // --- Var + Var ---
        impl Add<$var_type> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn add(self, other: Self) -> Self::Output {
                let mut terms = Vec::with_capacity(2);
                terms.push((self, 1.0));
                terms.push((other, 1.0));
                LinearExpr::with_terms(terms)
            }
        }
        forward_binop!(impl Add, add for $var_type, $var_type);

        // --- Var - Var ---
        impl Sub<$var_type> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn sub(self, other: Self) -> Self::Output {
                let mut terms = Vec::with_capacity(2);
                terms.push((self, 1.0));
                terms.push((other, -1.0));
                LinearExpr::with_terms(terms)
            }
        }
        forward_binop!(impl Sub, sub for $var_type, $var_type);

        // --- Var + Expr ---
        impl Add<LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn add(self, mut expr: LinearExpr<$var_type>) -> Self::Output {
                expr.add_term(self, 1.0);
                expr
            }
        }
        forward_binop!(impl Add, add for $var_type, LinearExpr<$var_type>);

        // --- Expr + Var ---
        impl Add<$var_type> for LinearExpr<$var_type> {
            type Output = Self;
            fn add(mut self, var: $var_type) -> Self {
                self.add_term(var, 1.0);
                self
            }
        }
        forward_binop!(impl Add, add for LinearExpr<$var_type>, $var_type);

        // --- Var - Expr ---
        // Logic: Var - Expr  =>  Var + (-1 * Expr)
        impl Sub<LinearExpr<$var_type>> for $var_type {
            type Output = LinearExpr<$var_type>;
            fn sub(self, mut expr: LinearExpr<$var_type>) -> Self::Output {
                expr.scale(-1.0);
                expr.add_term(self, 1.0);
                expr
            }
        }
        forward_binop!(impl Sub, sub for $var_type, LinearExpr<$var_type>);

        // --- Expr - Var ---
        impl Sub<$var_type> for LinearExpr<$var_type> {
            type Output = Self;
            fn sub(mut self, var: $var_type) -> Self {
                self.add_term(var, -1.0);
                self
            }
        }
        forward_binop!(impl Sub, sub for LinearExpr<$var_type>, $var_type);


        // ============================================================
        //  NUMERIC OPERATIONS (Generics)
        // ============================================================
        
        $(
            // --- Expr From Numeric ---
            impl From<$num_type> for LinearExpr<$var_type> {
                fn from(constant: $num_type) -> Self {
                    LinearExpr::with_constant(constant as f64)
                }
            }

            // --- Expr + Num ---
            impl Add<$num_type> for LinearExpr<$var_type> {
                type Output = Self;
                fn add(mut self, constant: $num_type) -> Self {
                    self.constant += constant as f64;
                    self
                }
            }
            forward_binop!(impl Add, add for LinearExpr<$var_type>, $num_type);

            // --- Num + Expr ---
            impl Add<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;
                fn add(self, mut expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    expr.constant += self as f64;
                    expr
                }
            }
            forward_binop!(impl Add, add for $num_type, LinearExpr<$var_type>);

            // --- Expr - Num ---
            impl Sub<$num_type> for LinearExpr<$var_type> {
                type Output = Self;
                fn sub(mut self, constant: $num_type) -> Self {
                    self.constant -= constant as f64;
                    self
                }
            }
            forward_binop!(impl Sub, sub for LinearExpr<$var_type>, $num_type);

            // --- Num - Expr ---
            impl Sub<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;
                fn sub(self, mut expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    expr.scale(-1.0);
                    expr.constant += self as f64;
                    expr
                }
            }
            forward_binop!(impl Sub, sub for $num_type, LinearExpr<$var_type>);

            // --- Expr * Num ---
            impl Mul<$num_type> for LinearExpr<$var_type> {
                type Output = Self;
                fn mul(mut self, constant: $num_type) -> Self {
                    self.scale(constant as f64);
                    self
                }
            }
            forward_binop!(impl Mul, mul for LinearExpr<$var_type>, $num_type);

            // --- Num * Expr ---
            impl Mul<LinearExpr<$var_type>> for $num_type {
                type Output = LinearExpr<$var_type>;
                fn mul(self, expr: LinearExpr<$var_type>) -> LinearExpr<$var_type> {
                    expr * self
                }
            }
            forward_binop!(impl Mul, mul for $num_type, LinearExpr<$var_type>);

            // --- Expr / Num ---
            impl Div<$num_type> for LinearExpr<$var_type> {
                type Output = Self;
                fn div(mut self, constant: $num_type) -> Self {
                    self.scale(1.0 / (constant as f64));
                    self
                }
            }
            forward_binop!(impl Div, div for LinearExpr<$var_type>, $num_type);

            // --- Var + Num ---
            impl Add<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;
                fn add(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = Vec::with_capacity(1);
                    terms.push((self, 1.0));
                    LinearExpr::with_terms_and_constant(terms, constant as f64)
                }
            }
            forward_binop!(impl Add, add for $var_type, $num_type);

            // --- Num + Var ---
            impl Add<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;
                fn add(self, var: $var_type) -> LinearExpr<$var_type> {
                    var + self
                }
            }
            forward_binop!(impl Add, add for $num_type, $var_type);

            // --- Var - Num ---
            impl Sub<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;
                fn sub(self, constant: $num_type) -> LinearExpr<$var_type> {
                    let mut terms = Vec::with_capacity(1);
                    terms.push((self, 1.0));
                    LinearExpr::with_terms_and_constant(terms, -(constant as f64))
                }
            }
            forward_binop!(impl Sub, sub for $var_type, $num_type);

            // --- Num - Var ---
            impl Sub<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;
                fn sub(self, var: $var_type) -> LinearExpr<$var_type> {
                    let mut terms = Vec::with_capacity(1);
                    terms.push((var, -1.0));
                    LinearExpr::with_terms_and_constant(terms, self as f64)
                }
            }
            forward_binop!(impl Sub, sub for $num_type, $var_type);

            // --- Var * Num ---
            impl Mul<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;
                fn mul(self, constant: $num_type) -> LinearExpr<$var_type> {
                    LinearExpr::with_term(self, constant as f64)
                }
            }
            forward_binop!(impl Mul, mul for $var_type, $num_type);

            // --- Num * Var ---
            impl Mul<$var_type> for $num_type {
                type Output = LinearExpr<$var_type>;
                fn mul(self, var: $var_type) -> LinearExpr<$var_type> {
                    LinearExpr::with_term(var, self as f64)
                }
            }
            forward_binop!(impl Mul, mul for $num_type, $var_type);

            // --- Var / Num ---
            impl Div<$num_type> for $var_type {
                type Output = LinearExpr<$var_type>;
                fn div(self, constant: $num_type) -> LinearExpr<$var_type> {
                    LinearExpr::with_term(self, 1.0 / (constant as f64))
                }
            }
            forward_binop!(impl Div, div for $var_type, $num_type);
        )*
    };
}

pub(crate) use impl_expr_display;
pub(crate) use impl_expr_ops;
