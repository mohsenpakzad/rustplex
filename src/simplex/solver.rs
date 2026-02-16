use std::{cmp, time::Instant};

use crate::{
    core::expression::LinearExpr,
    error::SolverError,
    standardization::{
        standard_model::StandardModel,
        standard_variable::StandardVariableKey
    },
    simplex::{
        slack::{
            dict_entry::DictEntryKey,
            dict_variable::{DictVariableKey, DictVariable},
            dictionary::SlackDictionary,
        },
        config::SolverConfig,
        solution::SolverSolution,
        status::SolverStatus,
    }
};

pub struct SimplexSolver {
    slack_dict: SlackDictionary,
    iteration_count: u32,
    config: SolverConfig,
}

impl SimplexSolver {
    pub fn form_standard_model(
        standard_model: &StandardModel,
        config: SolverConfig,
    ) -> Result<Self, SolverError> {
        if standard_model.variables().is_empty() {
            return Err(SolverError::NoVariables);
        } else if standard_model.objective().is_none() {
            return Err(SolverError::ObjectiveMissing);
        }

        let slack_dict = SlackDictionary::from_standard_model(standard_model);

        Ok(Self {
            slack_dict,
            iteration_count: 0,
            config,
        })
    }

    pub fn start(&mut self) -> SolverSolution<StandardVariableKey> {
        let start_time = Instant::now();
        if self.needs_phase_one() {
            let (aux_var_key, original_objective) = self.create_auxiliary_problem();

            let _phase1_status = self.solve_phase1(aux_var_key);

            // Phase1 is guaranteed that has a optimal solution,
            // so no check is needed
            if self.slack_dict.objective_value().abs() < *self.config.tolerance() {
                self.prepare_phase_two(aux_var_key, original_objective);
            } else {
                return SolverSolution::new_infeasible(self.iteration_count, start_time.elapsed());
            }
        }
        let phase2_status = self.solve();
        SolverSolution::new(
            phase2_status,
            self.slack_dict.objective_value(),
            self.slack_dict.std_values(),
            self.iteration_count,
            start_time.elapsed(),
        )
    }

    fn needs_phase_one(&self) -> bool {
        self.slack_dict
            .entries()
            .values()
            .any(|entry| entry.value() < self.config.neg_tolerance())
    }

    fn create_auxiliary_problem(&mut self) -> (DictVariableKey, LinearExpr<DictVariableKey>) {
        let aux_var_key = self.slack_dict.variables_mut().insert(DictVariable::new_auxiliary());

        let original_objective = self
            .slack_dict
            .replace_objective(LinearExpr::with_term(aux_var_key, -1.0));

        self.slack_dict.add_var_to_all_entries(aux_var_key, 1.0);

        (aux_var_key, original_objective)
    }

    fn prepare_phase_two(&mut self, aux_var: DictVariableKey, mut original_objective: LinearExpr<DictVariableKey>) {
        // 1. Check if the Auxiliary variable is still in the Basis
        // We look for an entry where the basic variable is 'Aux'
        let aux_entry = self
            .slack_dict
            .entries()
            .iter()
            .find(|(_, entry)| entry.basic_var() == aux_var);

        if let Some((entry_key, entry)) = aux_entry {
            // Try to find a non-basic variable in this row with a non-zero coefficient
            // to pivot with.
            let pivot_candidate = entry
                .expr()
                .terms
                .iter()
                .find(|&(_, coeff)| coeff.abs() > *self.config.tolerance())
                .map(|(var, _)| var.clone());

            if let Some(entering) = pivot_candidate {
                // Case A: Aux is basic, but we can pivot it out.
                // We pivot 'entering' INTO basis, and 'Aux' (entry) OUT of basis.
                self.slack_dict.pivot(entering, entry_key);
            } else {
                // Case B: Aux is basic, and implies 0 = 0 (redundant constraint).
                // All coefficients are zero. We can safely delete this row.
                self.slack_dict.remove_entry(entry_key);
            }
        }

        // 2. Now it is safe to remove Aux (it is guaranteed to be non-basic or gone)
        self.slack_dict.remove_var_from_all_entries(aux_var);

        // 3. Restore original objective
        self.slack_dict.entries().values().for_each(|entry| {
            original_objective.replace_var_with_expr(entry.basic_var(), &entry.expr());
        });
        self.slack_dict.set_objective(original_objective);
    }

    fn solve_phase1(&mut self, aux_var: DictVariableKey) -> SolverStatus {
        self.iteration_count += 1;
        let leaving = self.find_phase1_initial_leaving_variable();

        self.slack_dict.pivot(aux_var, leaving);

        self.solve()
    }

    fn solve(&mut self) -> SolverStatus {
        let max_iterations = self.config.max_iterations();
        while self.iteration_count < *max_iterations {
            self.iteration_count += 1;
            match self.find_entering_variable() {
                None => return SolverStatus::Optimal,
                Some(entering) => match self.find_leaving_variable(&entering) {
                    None => return SolverStatus::Unbounded,
                    Some(leaving) => {
                        self.slack_dict.pivot(entering, leaving);
                    }
                },
            };
        }
        SolverStatus::MaxIterationsReached
    }

    fn find_entering_variable(&self) -> Option<DictVariableKey> {
        self.slack_dict
            .objective()
            .terms
            .iter()
            .filter(|&(_, coefficient)| *coefficient > *self.config.tolerance())
            .max_by(|(v1, c1), (v2, c2)| {
                c1.total_cmp(c2) // Compare coefficients first
                    .then_with(|| self.compare_variables(v1, v2)) // Break ties by variable type
            })
            .map(|(var, _)| var.clone())
    }

    fn find_leaving_variable(&self, entering: &DictVariableKey) -> Option<DictEntryKey> {
        self.slack_dict
            .entries()
            .iter()
            .filter_map(|(entry_key, entry)| {
                let coefficient = entry.non_basic_coefficient(entering);
                if coefficient < self.config.neg_tolerance() {
                    Some((entry_key, entry, entry.value() / coefficient))
                } else {
                    None
                }
            })
            .max_by(|(_, e1, ub1), (_, e2, ub2)| {
                ub1.total_cmp(ub2) // Compare coefficients first
                    .then_with(|| self.compare_variables(&e1.basic_var(), &e2.basic_var())) // Break ties by variable type
            })
            .map(|(ek, _, _)| ek)
    }

    fn find_phase1_initial_leaving_variable(&self) -> DictEntryKey {
        self.slack_dict
            .entries()
            .iter()
            .min_by(|(_, e1), (_, e2)| e1.value().total_cmp(&e2.value()))
            .map(|(entry, _)| entry)
            .unwrap()
    }

    fn compare_variables(&self, var1: &DictVariableKey, var2: &DictVariableKey) -> cmp::Ordering {
        let var1 = self.slack_dict.variables().get(*var1).unwrap();
        let var2 = self.slack_dict.variables().get(*var2).unwrap();

        match (var1, var2) {
            (DictVariable::NonSlack(_), DictVariable::Slack(_)) => cmp::Ordering::Greater,
            (DictVariable::Auxiliary, DictVariable::Slack(_)) => cmp::Ordering::Greater,
            (DictVariable::Slack(_), DictVariable::NonSlack(_)) => cmp::Ordering::Less,
            (DictVariable::Slack(_), DictVariable::Auxiliary) => cmp::Ordering::Less,
            _ => cmp::Ordering::Equal,
        }
    }
}
