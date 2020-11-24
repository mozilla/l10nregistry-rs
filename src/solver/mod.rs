mod parallel;
mod serial;
mod solution;
pub mod testing;

pub use parallel::ParallelProblemSolver;
pub use serial::SerialProblemSolver;
pub use solution::Solution;

use crate::fluent::{FluentBundle, FluentResource};
use crate::registry::L10nRegistry;
use std::rc::Rc;
use unic_langid::LanguageIdentifier;

pub struct ProblemSolver {
    solution: Solution,

    cache: Vec<Vec<Option<Option<Rc<FluentResource>>>>>,

    langid: LanguageIdentifier,
    keys: Vec<String>,
    reg: L10nRegistry,
}

impl ProblemSolver {
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
                dirty: false,
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
}
