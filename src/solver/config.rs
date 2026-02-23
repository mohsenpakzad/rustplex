/// Configuration settings for the Simplex Solver.
///
/// This struct holds parameters that control the behavior of the optimization algorithm,
/// such as stopping criteria and numerical precision.
#[derive(Debug, Clone, Copy)]
pub struct SolverConfig {
    /// Maximum number of iterations before stopping (Default: 10000).
    pub max_iterations: u32,
    /// Numerical tolerance for floating-point comparisons (Default: 1e-10).
    pub tolerance: f64,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10_000,
            tolerance: 1e-10,
        }
    }
}
