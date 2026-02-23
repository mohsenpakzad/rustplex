use std::fmt;

/// Represents the terminal state of the solver execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverStatus {
    /// A solution satisfying all constraints and optimizing the objective was found.
    Optimal,
    /// No solution satisfies all constraints.
    Infeasible,
    /// The objective function can be improved indefinitely (it goes to infinity).
    Unbounded,
    /// The solver reached the maximum allowed iterations without finding a definitive result.
    /// The returned solution is the best found so far (or the last valid basis).
    MaxIterationsReached,
}

impl SolverStatus {
    /// Returns true if the status represents a successful optimal solution.
    pub fn is_optimal(&self) -> bool {
        matches!(self, Self::Optimal)
    }

    /// Returns a human-readable description of the status.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Optimal => "Optimal solution found",
            Self::Infeasible => "Problem is infeasible",
            Self::Unbounded => "Problem is unbounded",
            Self::MaxIterationsReached => "Maximum iteration limit reached"
        }
    }
}

impl fmt::Display for SolverStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}
