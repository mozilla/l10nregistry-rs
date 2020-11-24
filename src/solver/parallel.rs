use super::ProblemSolver;
use crate::fluent::FluentBundle;
use crate::registry::L10nRegistry;
use std::ops::{Deref, DerefMut};
use unic_langid::LanguageIdentifier;

pub struct ParallelProblemSolver {
    solver: ProblemSolver,
}

impl Deref for ParallelProblemSolver {
    type Target = ProblemSolver;

    fn deref(&self) -> &Self::Target {
        &self.solver
    }
}

impl DerefMut for ParallelProblemSolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.solver
    }
}

impl ParallelProblemSolver {
    pub fn new(keys: Vec<String>, langid: LanguageIdentifier, reg: L10nRegistry) -> Self {
        Self {
            solver: ProblemSolver::new(keys, langid, reg),
        }
    }

    async fn test_complete_solution(&mut self) -> Result<(), usize> {
        let set = self
            .reg
            .lock()
            .generate_resource_set(&self.langid, &self.solution.candidate, &self.keys)
            .await;

        if let Err(idx) = set {
            Err(idx)
        } else {
            Ok(())
        }
    }

    pub async fn next(&mut self) -> Option<&Vec<usize>> {
        if self.solution.dirty {
            if !self.solution.bail() {
                return None;
            }
            self.solution.dirty = false;
        }
        while self.solution.advance_to_completion() {
            if let Err(idx) = self.test_complete_solution().await {
                self.solution.idx = idx;
                self.solution.prune();
                if !self.solution.bail() {
                    return None;
                }
                continue;
            }
            self.solution.dirty = true;
            return Some(&self.solution.candidate);
        }
        None
    }

    pub async fn next_bundle(&mut self) -> Option<FluentBundle> {
        if self.solution.dirty {
            if !self.solution.bail() {
                return None;
            }
            self.solution.dirty = false;
        }
        while self.solution.advance_to_completion() {
            if let Err(idx) = self.test_complete_solution().await {
                self.solution.idx = idx;
                self.solution.prune();
                if !self.solution.bail() {
                    return None;
                }
                continue;
            }
            self.solution.dirty = true;
            return Some(self.get_bundle());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::testing::get_scenarios;
    use unic_langid::LanguageIdentifier;

    #[tokio::test]
    async fn test_async() {
        let scenarios = get_scenarios();

        let langid: LanguageIdentifier = "en-US".parse().unwrap();

        for scenario in scenarios {
            let reg = scenario.get_l10nregistry();
            let mut gen = ParallelProblemSolver::new(scenario.res_ids.clone(), langid.clone(), reg);

            if let Some(solutions) = &scenario.solutions {
                let mut i = 0;
                while let Some(solution) = gen.next().await {
                    assert!(solutions.len() > i);
                    assert_eq!(solution, solutions.get(i).unwrap());
                    i += 1;
                }
                assert_eq!(i, solutions.len());
            }
        }
    }
}
