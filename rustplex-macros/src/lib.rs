mod expr;

use expr::expr_to_linear;
use proc_macro::TokenStream;
use syn::{parse_macro_input, Expr};

/// A proc macro that converts Rust expressions into LinearExpr instances
#[proc_macro]
pub fn expr(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    expr_to_linear(&expr).into()
}
