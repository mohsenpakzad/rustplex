use std::fmt;

use crate::{
    core::{
        constraint::{ConstrRef, ConstraintSense},
        expression::LinearExpr,
        objective::{Objective, ObjectiveSense},
        variable::VarRef,
    },
    standardization::standard_model::StandardModel,
};

#[derive(Debug, Default)]
pub struct Model {
    variables: Vec<VarRef>,
    constraints: Vec<ConstrRef>,
    objective: Option<Objective>,
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_variable(&mut self) -> VarRef {
        let var = VarRef::new();
        self.variables.push(var.clone());
        var
    }

    pub fn add_constraint(
        &mut self,
        lhs: impl Into<LinearExpr<VarRef>>,
        sense: ConstraintSense,
        rhs: impl Into<LinearExpr<VarRef>>,
    ) -> ConstrRef {
        let constr = ConstrRef::new(lhs.into(), sense, rhs.into());
        self.constraints.push(constr.clone());
        constr
    }

    pub fn set_objective(
        &mut self,
        sense: ObjectiveSense,
        expression: impl Into<LinearExpr<VarRef>>,
    ) {
        self.objective = Some(Objective::new(sense, expression.into()));
    }

    pub fn to_standard(&self) -> StandardModel {
        StandardModel::from_model(&self)
    }

    pub fn get_variables(&self) -> &Vec<VarRef> {
        &self.variables
    }

    pub fn get_constraints(&self) -> &Vec<ConstrRef> {
        &self.constraints
    }

    pub fn get_objective(&self) -> &Option<Objective> {
        &self.objective
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
