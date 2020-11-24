use super::ProblemSolver;
use crate::fluent::FluentBundle;
use crate::registry::L10nRegistry;
use std::ops::{Deref, DerefMut};
use unic_langid::LanguageIdentifier;

pub struct SerialProblemSolver {
    solver: ProblemSolver,
}

impl Deref for SerialProblemSolver {
    type Target = ProblemSolver;

    fn deref(&self) -> &Self::Target {
        &self.solver
    }
}

impl DerefMut for SerialProblemSolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.solver
    }
}

impl SerialProblemSolver {
    pub fn new(keys: Vec<String>, langid: LanguageIdentifier, reg: L10nRegistry) -> Self {
        Self {
            solver: ProblemSolver::new(keys, langid, reg),
        }
    }

    #[inline]
    pub fn next(&mut self) -> Option<&Vec<usize>> {
        if self.solution.dirty {
            if !self.solution.bail() {
                return None;
            }
            self.solution.dirty = false;
        }
        loop {
            if !self.test_solution() {
                if !self.solution.bail() {
                    return None;
                }
                continue;
            }
            if self.solution.is_complete() {
                self.solution.dirty = true;
                return Some(&self.solution.candidate);
            }
            if !self.solution.advance() {
                return None;
            }
        }
    }

    #[inline]
    pub fn next_bundle(&mut self) -> Option<FluentBundle> {
        if self.solution.dirty {
            if !self.solution.bail() {
                return None;
            }
            self.solution.dirty = false;
        }
        loop {
            if !self.test_solution() {
                if !self.solution.bail() {
                    return None;
                }
                continue;
            }
            if self.solution.is_complete() {
                self.solution.dirty = true;
                return Some(self.get_bundle());
            }
            if !self.solution.advance() {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::testing::get_scenarios;
    use unic_langid::LanguageIdentifier;

    #[test]
    fn test_sync() {
        let scenarios = get_scenarios();

        let langid: LanguageIdentifier = "en-US".parse().unwrap();

        for scenario in scenarios {
            let reg = scenario.get_l10nregistry();
            let mut gen = SerialProblemSolver::new(scenario.res_ids.clone(), langid.clone(), reg);

            if let Some(solutions) = &scenario.solutions {
                let mut i = 0;
                while let Some(solution) = gen.next() {
                    assert!(
                        solutions.len() > i,
                        "too many solutions, scenario: {}",
                        scenario.name
                    );
                    assert_eq!(solution, solutions.get(i).unwrap());
                    i += 1;
                }
                assert_eq!(
                    i,
                    solutions.len(),
                    "too few solutions, scenario: {}",
                    scenario.name
                );
            }
        }
    }
}
