use crate::core::{
    constraint::{ConstrRef, ConstraintSense},
    expression::LinearExpr,
    objective::{Objective, ObjectiveSense},
    variable::VarRef,
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

    // pub fn solve(&self) -> Result<Vec<f64>, crate::error::LPError> {
    //     // Implement solver logic here
    //     // Return the values of all variables in order
    //     Ok(vec![])
    // }

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
