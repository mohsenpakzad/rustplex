use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolverError {
    #[error("Non-linear programming is not supported yet.")]
    NonLinearNotSupported,

    #[error("Objective function must be set before solving.")]
    ObjectiveMissing,
}