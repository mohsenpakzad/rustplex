use core::panic;
use std::{cmp, time::Instant};

use crate::{
    core::expression::LinearExpr,
    standardization::{standard_model::StandardModel, standard_variable::StdVar},
};

use super::{
    config::SolverConfig,
    slack::{
        dict_entry::DictEntryRef,
        dict_variable::{DictVar, DictVariable},
        dictionary::SlackDictionary,
    },
    solution::SolverSolution,
    status::SolverStatus,
};

pub struct SimplexSolver {
    slack_dict: SlackDictionary,
    iteration_count: u32,
    config: SolverConfig,
}

impl SimplexSolver {
    pub fn form_standard_model(standard_model: &StandardModel, config: SolverConfig) -> Self {
        if standard_model.get_objective().is_none() {
            panic!("Objective must be set.");
        }

        let slack_dict = SlackDictionary::from_standard_model(standard_model);

        Self {
            slack_dict,
            iteration_count: 0,
            config,
        }
    }

    pub fn start(&mut self) -> SolverSolution<StdVar> {
        let start_time = Instant::now();
        if self.needs_phase_one() {
            let (aux_var, original_objective) = self.create_auxiliary_problem();

            let _phase1_status = self.solve_phase1(aux_var.clone());

            // Phase1 is guaranteed that has a optimal solution,
            // so no check is needed
            if self.slack_dict.get_objective_value().abs() < *self.config.tolerance() {
                self.prepare_phase_two(aux_var, original_objective);
            } else {
                return SolverSolution::new_infeasible(self.iteration_count, start_time.elapsed());
            }
        }
        let phase2_status = self.solve();
        SolverSolution::new(
            phase2_status,
            self.slack_dict.get_objective_value(),
            self.slack_dict.get_std_values(),
            self.iteration_count,
            start_time.elapsed(),
        )
    }

    fn needs_phase_one(&self) -> bool {
        self.slack_dict
            .get_entires()
            .iter()
            .any(|entry| entry.get_value() < self.config.neg_tolerance())
    }

    fn create_auxiliary_problem(&mut self) -> (DictVar, LinearExpr<DictVar>) {
        let aux_var = DictVar::new_non_slack(StdVar::default().name("Aux"));

        let original_objective = self
            .slack_dict
            .replace_objective(LinearExpr::with_term(aux_var.clone(), -1.0));

        self.slack_dict.add_var_to_all_entries(aux_var.clone(), 1.0);

        (aux_var, original_objective)
    }

    fn prepare_phase_two(
        &mut self,
        aux_var: DictVar,
        mut original_objective: LinearExpr<DictVar>,
    ) {
        self.slack_dict.remove_var_from_all_entries(aux_var);

        self.slack_dict.get_entires().iter().for_each(|entry| {
            original_objective.replace_var_with_expr(entry.get_basic_var(), &entry.get_expr());
        });
        self.slack_dict.set_objective(original_objective);
    }

    fn solve_phase1(&mut self, aux_var: DictVar) -> SolverStatus {
        self.iteration_count += 1;
        let leaving = self.find_phase1_initial_leaving_variable();

        self.slack_dict.pivot(&aux_var, &leaving);

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
                        self.slack_dict.pivot(&entering, &leaving);
                    }
                },
            };
        }
        SolverStatus::MaxIterationsReached
    }

    fn find_entering_variable(&self) -> Option<DictVar> {
        self.slack_dict
            .get_objective()
            .terms
            .iter()
            .filter(|(_, &coefficient)| coefficient > *self.config.tolerance())
            .max_by(|(v1, c1), (v2, c2)| {
                c1.total_cmp(c2) // Compare coefficients first
                    .then_with(|| Self::compare_variables(v1, v2)) // Break ties by variable type
            })
            .map(|(var, _)| var.clone())
    }

    fn find_leaving_variable(&self, entering: &DictVar) -> Option<DictEntryRef> {
        self.slack_dict
            .get_entires()
            .iter()
            .filter_map(|entry| {
                let coefficient = entry.get_non_basic_coefficient(entering);
                if coefficient < self.config.neg_tolerance() {
                    Some((entry.clone(), entry.get_value() / coefficient))
                } else {
                    None
                }
            })
            .max_by(|(e1, ub1), (e2, ub2)| {
                ub1.total_cmp(ub2) // Compare coefficients first
                    .then_with(|| Self::compare_variables(&e1.get_basic_var(), &e2.get_basic_var()))
                // Break ties by variable type
            })
            .map(|(e, _)| e)
    }

    fn find_phase1_initial_leaving_variable(&self) -> DictEntryRef {
        self.slack_dict
            .get_entires()
            .iter()
            .min_by(|e1, e2| e1.get_value().total_cmp(&e2.get_value()))
            .map(|entry| entry.clone())
            .unwrap()
    }

    fn compare_variables(var1: &DictVar, var2: &DictVar) -> cmp::Ordering {
        match (var1.get_var(), var2.get_var()) {
            (DictVariable::NonSlack(_), DictVariable::Slack(_)) => cmp::Ordering::Greater,
            (DictVariable::Slack(_), DictVariable::NonSlack(_)) => cmp::Ordering::Less,
            _ => cmp::Ordering::Equal,
        }
    }
}
