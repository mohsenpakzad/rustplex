use proc_macro2::TokenStream;
use quote::quote;
use syn::{BinOp, Expr};

/// Main expression parser that converts Syn::Expr to TokenStream
pub fn expr_to_linear(expr: &Expr) -> TokenStream {
    match expr {
        Expr::Binary(bin_expr) => parse_binary_expr(bin_expr),
        Expr::Path(path) => parse_variable(path),
        Expr::Lit(lit) => parse_constant(lit),
        _ => panic!("Unsupported expression type in linear expression"),
    }
}

/// Handle binary operations (+, -, *, /)
fn parse_binary_expr(bin_expr: &syn::ExprBinary) -> TokenStream {
    let lhs = expr_to_linear(&bin_expr.left);
    let rhs = expr_to_linear(&bin_expr.right);

    match &bin_expr.op {
        BinOp::Add(_) => generate_addition(lhs, rhs),
        BinOp::Sub(_) => generate_subtraction(lhs, rhs),
        BinOp::Mul(_) => generate_multiplication(&bin_expr.left, &bin_expr.right),
        BinOp::Div(_) => generate_division(&bin_expr.left, &bin_expr.right),
        _ => panic!("Unsupported operator in linear expression"),
    }
}

/// Generate code for addition
fn generate_addition(lhs: TokenStream, rhs: TokenStream) -> TokenStream {
    quote!({
        let mut result = #lhs;
        result.add_expr(&#rhs);
        result
    })
}

/// Generate code for subtraction
fn generate_subtraction(lhs: TokenStream, rhs: TokenStream) -> TokenStream {
    quote!({
        let mut result = #lhs;
        result.sub_expr(&#rhs);
        result
    })
}

/// Generate code for multiplication, ensuring linearity is maintained
fn generate_multiplication(left: &Expr, right: &Expr) -> TokenStream {
    match (left, right) {
        // Constant * Variable or Variable * Constant
        (Expr::Lit(lit), Expr::Path(var)) | (Expr::Path(var), Expr::Lit(lit)) => {
            quote!({
                let mut expr = LinearExpr::new();
                expr.add_term(#var.clone(), #lit as f64);
                expr
            })
        }
        // Constant * Constant
        (Expr::Lit(lit1), Expr::Lit(lit2)) => {
            quote!(LinearExpr::with_constant((#lit1 as f64) * (#lit2 as f64)))
        }
        _ => panic!("Unsupported multiplication in linear expression"),
    }
}

/// Generate code for division, ensuring linearity is maintained
fn generate_division(left: &Expr, right: &Expr) -> TokenStream {
    match (left, right) {
        // Variable / Constant
        (Expr::Path(var), Expr::Lit(lit)) => {
            quote!({
                let divisor = #lit as f64;
                if divisor == 0.0 {
                    panic!("Division by zero in linear expression");
                }
                let mut expr = LinearExpr::new();
                expr.add_term(#var.clone(), 1.0 / divisor);
                expr
            })
        }
        // Constant / Constant
        (Expr::Lit(lit1), Expr::Lit(lit2)) => {
            quote!({
                let divisor = #lit2 as f64;
                if divisor == 0.0 {
                    panic!("Division by zero in linear expression");
                }
                LinearExpr::with_constant((#lit1 as f64) / divisor)
            })
        }
        _ => panic!("Unsupported division in linear expression"),
    }
}

/// Generate code for a single variable
fn parse_variable(path: &syn::ExprPath) -> TokenStream {
    quote!({
        let mut expr = LinearExpr::new();
        expr.add_term(#path.clone(), 1.0);
        expr
    })
}

/// Generate code for a constant value
fn parse_constant(lit: &syn::ExprLit) -> TokenStream {
    quote!(LinearExpr::with_constant(#lit as f64))
}
