use std::fmt;

use crate::{
    core::{
        constraint::{Constr, ConstraintSense},
        expression::LinearExpr,
        objective::{Objective, ObjectiveSense},
        variable::Var,
    },
    simplex::{config::SolverConfig, solution::SolverSolution},
    standardization::standard_model::StandardModel,
};

use super::variable::VariableType;

#[derive(Debug, Default)]
pub struct Model {
    variables: Vec<Var>,
    constraints: Vec<Constr>,
    objective: Option<Objective>,
    solution: SolverSolution<Var>,
    config: Option<SolverConfig>,
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: SolverConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn add_variable(&mut self) -> Var {
        let var = Var::new();
        self.variables.push(var.clone());
        var
    }

    pub fn add_constraint(
        &mut self,
        lhs: impl Into<LinearExpr<Var>>,
        sense: ConstraintSense,
        rhs: impl Into<LinearExpr<Var>>,
    ) -> Constr {
        let constr = Constr::new(lhs.into(), sense, rhs.into());
        self.constraints.push(constr.clone());
        constr
    }

    pub fn set_objective(&mut self, sense: ObjectiveSense, expression: impl Into<LinearExpr<Var>>) {
        self.objective = Some(Objective::new(sense, expression.into()));
    }

    pub fn is_lp(&self) -> bool {
        !self
            .variables
            .iter()
            .any(|var| !matches!(var.get_type(), VariableType::Continuous))
    }

    pub fn to_standard(&self) -> StandardModel {
        StandardModel::from_model(&self)
    }

    pub fn solve(&mut self) {
        if !self.is_lp() {
            todo!("Non LP calculation is not supported yet.")
        }

        let mut standardized_model =
            StandardModel::from_model(&self).with_config(self.config.clone().unwrap_or_default());
        standardized_model.solve();
        self.solution = standardized_model.get_model_solution().unwrap()
    }

    pub fn get_variables(&self) -> &Vec<Var> {
        &self.variables
    }

    pub fn get_constraints(&self) -> &Vec<Constr> {
        &self.constraints
    }

    pub fn get_objective(&self) -> &Option<Objective> {
        &self.objective
    }

    pub fn get_solution(&self) -> &SolverSolution<Var> {
        &self.solution
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display the objective, if it exists
        match &self.objective {
            Some(objective) => {
                writeln!(f, "Objective: {}", objective)?;
            }
            None => {
                writeln!(f, "Objective: None")?;
            }
        }

        // Display the constraints
        writeln!(f, "Constraints: [")?;
        for constr in self.constraints.iter() {
            writeln!(f, "\t{},", constr)?;
        }
        writeln!(f, "]")?;

        // Display the variables
        writeln!(f, "Variables: [")?;
        for var in self.variables.iter() {
            writeln!(f, "\t{},", var)?;
        }
        write!(f, "]")?;

        Ok(())
    }
}
