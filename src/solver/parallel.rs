use super::solution::Solution;
use crate::fluent::{FluentBundle, FluentResource};
use crate::registry::L10nRegistry;
use std::rc::Rc;
use unic_langid::LanguageIdentifier;

pub struct ParallelProblemSolver {
    solution: Solution,

    cache: Vec<Vec<Option<Option<Rc<FluentResource>>>>>,

    langid: LanguageIdentifier,
    keys: Vec<String>,
    reg: L10nRegistry,
}

impl ParallelProblemSolver {
    pub fn new(keys: Vec<String>, langid: LanguageIdentifier, reg: L10nRegistry) -> Self {
        let width = keys.len();
        let depth = reg.shared.sources.borrow().len();
        debug_assert!(width > 0);
        debug_assert!(depth > 0);

        Self {
            solution: Solution {
                width,
                depth,
                candidate: vec![0; width],
                idx: 0,
            },

            cache: vec![vec![None; depth]; width],

            langid,
            keys,
            reg,
        }
    }

    #[inline]
    fn fetch_cell(&mut self, res_idx: usize, source_idx: usize) -> bool {
        let cell = &mut self.cache[res_idx][source_idx];

        if let Some(val) = cell {
            val.is_some()
        } else {
            let key = &self.keys[res_idx];
            let file = self
                .reg
                .lock()
                .get_file_from_source(&self.langid, source_idx, key);
            let result = file.is_some();
            *cell = Some(file);
            result
        }
    }

    #[inline]
    fn get_bundle(&self) -> FluentBundle {
        let mut bundle = FluentBundle::new(&[self.langid.clone()]);
        for (res_idx, source_idx) in self.solution.candidate.iter().enumerate() {
            let cell = &self.cache[res_idx][*source_idx];
            bundle
                .add_resource(cell.as_ref().unwrap().as_ref().unwrap().clone())
                .unwrap()
        }
        return bundle;
    }

    fn test_solution(&mut self) -> bool {
        let res_idx = self.solution.idx;
        let source_idx = self.solution.candidate[res_idx];
        self.fetch_cell(res_idx, source_idx)
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
        if self.solution.is_complete() {
            if !self.solution.bail() {
                return None;
            }
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
            return Some(&self.solution.candidate);
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
