use std::fmt;

#[derive(Debug, Clone)]
pub struct LinearExpr<T: ExprVariable> {
    pub terms: Vec<(T, f64)>,
    pub constant: f64,
}

pub trait ExprVariable: Clone + Eq + fmt::Display {}

impl<T: ExprVariable> LinearExpr<T> {
    pub fn new() -> Self {
        Self {
            terms: Vec::new(),
            constant: 0.0,
        }
    }

    pub fn with_term(var: T, coefficient: f64) -> Self {
        let mut terms = Vec::with_capacity(1);
        terms.push((var, coefficient));
        Self {
            terms,
            constant: 0.0,
        }
    }

    pub fn with_terms(terms: Vec<(T, f64)>) -> Self {
        Self {
            terms,
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
        Self { terms, constant }
    }

    pub fn coefficient(&self, var: &T) -> f64 {
        self.terms
            .iter()
            .find(|(v, _)| v == var)
            .map(|(_, c)| *c)
            .unwrap_or(0.0)
    }

    pub fn add_term(&mut self, var: T, coefficient: f64) {
        if let Some((_, c)) = self.terms.iter_mut().find(|(v, _)| *v == var) {
            *c += coefficient;
        } else {
            self.terms.push((var, coefficient));
        }
    }

    pub fn remove_term(&mut self, var: &T) -> Option<f64> {
        if let Some(idx) = self.terms.iter().position(|(v, _)| v == var) {
            let (_, coeff) = self.terms.swap_remove(idx);
            Some(coeff)
        } else {
            None
        }
    }

    pub fn add_expr(&mut self, other: &Self) {
        for (var, coefficient) in &other.terms {
            self.add_term(var.clone(), *coefficient);
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
        for (_, coefficient) in self.terms.iter_mut() {
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
        use crate::modeling::expression::LinearExpr;

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
