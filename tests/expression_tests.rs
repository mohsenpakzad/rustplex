use rustplex::{
    core::expression::LinearExpr,
    Var,
};

/// Test: Term Aggregation
/// Logic: 1.0*x + 2.0*x should become 3.0*x
#[test]
fn test_add_same_variable() {
    let x = Var::new().with_name("x");
    
    // x + 2x
    let expr: LinearExpr<Var> = LinearExpr::from(&x) + (LinearExpr::from(&x) * 2.0);
    
    assert_eq!(expr.coefficient(&x), 3.0);
    assert_eq!(expr.terms.len(), 1); // Should merge into one term
}

/// Test: Subtraction and Cancellation
/// Logic: 3x - 3x should result in 0 coefficient
#[test]
fn test_term_cancellation() {
    let x = Var::new().with_name("x");
    
    let expr: LinearExpr<Var> = (LinearExpr::from(&x) * 3.0) - (LinearExpr::from(&x) * 3.0);
    
    // Depending on implementation details, it might be 0.0 or removed.
    // Checking coefficient is safer.
    assert_eq!(expr.coefficient(&x), 0.0);
    assert_eq!(expr.constant, 0.0);
}

/// Test: Constant Algebra
/// Logic: (x + 5) + (y - 2) should be x + y + 3
#[test]
fn test_constant_math() {
    let x = Var::new().with_name("x");
    let y = Var::new().with_name("y");
    
    // (x + 5)
    let expr1 = LinearExpr::from(&x) + 5.0;
    // (y - 2)
    let expr2 = LinearExpr::from(&y) - 2.0;
    
    let sum: LinearExpr<Var> = expr1 + expr2;
    
    assert_eq!(sum.coefficient(&x), 1.0);
    assert_eq!(sum.coefficient(&y), 1.0);
    assert_eq!(sum.constant, 3.0); // 5 - 2 = 3
}

/// Test: Scalar Multiplication
/// Logic: 2 * (3x + 4) should be 6x + 8
#[test]
fn test_distributive_property() {
    let x = Var::new().with_name("x");
    
    let expr = LinearExpr::from(&x) * 3.0 + 4.0;
    let scaled: LinearExpr<Var> = expr * 2.0;
    
    assert_eq!(scaled.coefficient(&x), 6.0);
    assert_eq!(scaled.constant, 8.0);
}

/// Test: Complex Linear Combination
/// Logic: 2x - (y + x) = x - y
#[test]
fn test_complex_combination() {
    let x = Var::new().with_name("x");
    let y = Var::new().with_name("y");

    let expr: LinearExpr<Var> = (LinearExpr::from(&x) * 2.0) - (LinearExpr::from(&y) + &x);

    assert_eq!(expr.coefficient(&x), 1.0);
    assert_eq!(expr.coefficient(&y), -1.0);
}