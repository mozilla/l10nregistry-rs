mod parallel;
mod serial;
mod solution;
pub mod testing;

pub use parallel::ParallelProblemSolver;
pub use serial::SerialProblemSolver;
pub use solution::Solution;

use crate::fluent::FluentBundle;
use crate::registry::L10nRegistry;
use unic_langid::LanguageIdentifier;

pub struct ProblemSolver {
    solution: Solution,

    langid: LanguageIdentifier,
    keys: Vec<String>,
    reg: L10nRegistry,
}

impl ProblemSolver {
    pub fn new(keys: Vec<String>, langid: LanguageIdentifier, reg: L10nRegistry) -> Self {
        let width = keys.len();
        let depth = reg.shared.sources.borrow().len();
        Self {
            solution: Solution {
                width,
                depth,
                candidate: vec![0; width],
                idx: 0,
                dirty: false,

                cache: vec![vec![None; depth]; width],
            },

            langid,
            keys,
            reg,
        }
    }

    fn get_bundle(&self) -> FluentBundle {
        let mut bundle = FluentBundle::new(&[self.langid.clone()]);
        for (res_idx, source_idx) in self.solution.candidate.iter().enumerate() {
            let cell = &self.solution.cache[res_idx][*source_idx];
            bundle
                .add_resource(cell.as_ref().unwrap().as_ref().unwrap().clone())
                .unwrap()
        }
        bundle
    }
}
