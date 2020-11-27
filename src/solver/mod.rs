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

                cache: vec![vec![None; depth]; width],
            },

            langid,
            keys,
            reg,
        }
    }
}
