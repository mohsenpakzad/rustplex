use std::rc::Rc;

#[derive(Debug, Clone)]
struct SolverConfiguration {
    /// Maximum number of iterations before stopping
    max_iterations: u32,
    /// Numerical tolerance for floating-point comparisons
    tolerance: f64,
}

impl Default for SolverConfiguration {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            tolerance: 1e-10,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SolverConfig(Rc<SolverConfiguration>);

impl SolverConfig {
    pub fn new(max_iterations: u32, tolerance: f64) -> Self {
        Self(Rc::new(SolverConfiguration {
            max_iterations,
            tolerance,
        }))
    }

    pub fn max_iterations(&self) -> &u32 {
        &self.0.max_iterations
    }

    pub fn tolerance(&self) -> &f64 {
        &self.0.tolerance
    }

    pub fn neg_tolerance(&self) -> f64 {
        -self.0.tolerance
    }
}
