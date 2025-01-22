#[derive(Debug, Clone)]
pub struct SolverConfig {
    /// Maximum number of iterations before stopping
    max_iterations: u32,
    /// Numerical tolerance for floating-point comparisons
    tolerance: f64,
}

impl SolverConfig {
    pub fn new(max_iterations: u32, tolerance: f64) -> Self {
        Self {
            max_iterations,
            tolerance,
        }
    }

    pub fn with_max_iterations(max_iterations: u32) -> Self {
        Self {
            max_iterations,
            tolerance: 1e-10,
        }
    }

    pub fn with_tolerance(tolerance: f64) -> Self {
        Self {
            max_iterations: 1000,
            tolerance,
        }
    }

    pub fn max_iterations(&self) -> &u32 {
        &self.max_iterations
    }

    pub fn tolerance(&self) -> &f64 {
        &self.tolerance
    }

    pub fn neg_tolerance(&self) -> f64 {
        -self.tolerance
    }
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            tolerance: 1e-10,
        }
    }
}
