pub const EPSILON: f64 = 1e-6;

/// Asserts that two floats are approximately equal.
///
/// # Arguments
/// * `actual` - The value calculated by the solver.
/// * `expected` - The correct value expected.
pub fn assert_approx_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < EPSILON,
        "Test Failed: Expected {}, but got {}",
        expected,
        actual
    );
}
