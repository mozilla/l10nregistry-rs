use super::ProblemSolver;
use std::ops::{Deref, DerefMut};

pub trait SyncTester {
    fn test_sync(&self, res_idx: usize, source_idx: usize) -> bool;
}

pub struct SerialProblemSolver(ProblemSolver);

impl Deref for SerialProblemSolver {
    type Target = ProblemSolver;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SerialProblemSolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SerialProblemSolver {
    pub fn new(width: usize, depth: usize) -> Self {
        Self(ProblemSolver::new(width, depth))
    }
}

impl SerialProblemSolver {
    fn test_current_cell<T>(&mut self, tester: &T) -> bool
    where
        T: SyncTester,
    {
        let res_idx = self.idx;
        let source_idx = self.solution[res_idx];
        let cell = &self.cache[res_idx][source_idx];
        if let Some(val) = cell {
            *val
        } else {
            tester.test_sync(res_idx, source_idx)
        }
    }

    pub fn next<T>(&mut self, tester: &T) -> Option<&[usize]>
    where
        T: SyncTester,
    {
        if self.width == 0 || self.depth == 0 {
            return None;
        }
        if self.dirty {
            if !self.bail() {
                return None;
            }
            self.dirty = false;
        }
        loop {
            if !self.test_current_cell(tester) {
                if !self.bail() {
                    return None;
                }
                continue;
            }
            if self.is_complete() {
                self.dirty = true;
                return Some(&self.solution);
            }
            if !self.try_advance_resource() {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn problem_solver() {
        // let keys = vec!["key1.ftl", "key2.ftl"];
        // let sources = vec!["source1", "source2"];
        // let args = ("foo",);

        // let ps = ProblemSolver::new(keys.len(), sources.len(), &foo);
    }
}
