// src/solver/status.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverStatus {
    Optimal,
    Infeasible,
    Unbounded,
    NotSolved,
    MaxIterationsReached,
    // Numerical, // For numerical stability issues
    // IllFormed, // For problems with invalid input
}

impl SolverStatus {
    pub fn is_optimal(&self) -> bool {
        matches!(self, SolverStatus::Optimal)
    }

    pub fn description(&self) -> &'static str {
        match self {
            SolverStatus::Optimal => "Optimal solution found",
            SolverStatus::Infeasible => "Problem is infeasible",
            SolverStatus::Unbounded => "Problem is unbounded",
            SolverStatus::NotSolved => "Problem has not been solved",
            SolverStatus::MaxIterationsReached => "Maximum iterations reached",
            // SolverStatus::Numerical => "Numerical difficulties encountered",
            // SolverStatus::IllFormed => "Problem is ill-formed",
        }
    }
}
