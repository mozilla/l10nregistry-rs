use crate::fluent::FluentResource;
use std::rc::Rc;

#[derive(Debug)]
pub struct Solution {
    pub width: usize,
    pub depth: usize,
    pub candidate: Vec<usize>,
    pub idx: usize,
    pub dirty: bool,

    pub cache: Vec<Vec<Option<Option<Rc<FluentResource>>>>>,
}

impl Solution {
    pub fn get_cell(
        &self,
        res_idx: usize,
        source_idx: usize,
    ) -> &Option<Option<Rc<FluentResource>>> {
        &self.cache[res_idx][source_idx]
    }

    fn is_cell_missing(&self, res_idx: usize, source_idx: usize) -> bool {
        if let Some(None) = self.cache[res_idx][source_idx] {
            return true;
        }
        false
    }

    fn is_current_cell_missing(&self) -> bool {
        let res_idx = self.idx;
        let source_idx = self.candidate[res_idx];
        let cell = &self.cache[res_idx][source_idx];
        if let Some(None) = cell {
            return true;
        }
        false
    }

    pub fn try_advance_source(&mut self) -> bool {
        while self.candidate[self.idx] < self.depth - 1 {
            self.candidate[self.idx] += 1;
            if !self.is_current_cell_missing() {
                return true;
            }
        }
        false
    }

    pub fn try_advance_resource(&mut self) -> bool {
        if self.idx >= self.width - 1 {
            false
        } else {
            self.idx += 1;
            while self.is_current_cell_missing() {
                if !self.try_advance_source() {
                    return false;
                }
            }
            true
        }
    }

    pub fn try_backtrack(&mut self) -> bool {
        while self.candidate[self.idx] == self.depth - 1 {
            if self.idx == 0 {
                return false;
            }
            self.idx -= 1;
        }
        self.candidate[self.idx] += 1;
        self.prune()
    }

    pub fn prune(&mut self) -> bool {
        for i in self.idx + 1..self.width {
            let mut source_idx = 0;
            while self.is_cell_missing(i, source_idx) {
                if source_idx >= self.depth - 1 {
                    return false;
                }
                source_idx += 1;
            }
            // println!("prune new candidate: {}, {}", i, source_idx);
            self.candidate[i] = source_idx;
        }
        // println!("prune: {:?}", self.candidate);
        true
    }

    pub fn is_complete(&self) -> bool {
        self.idx == self.width - 1
    }

    pub fn bail(&mut self) -> bool {
        if self.try_advance_source() {
            true
        } else {
            self.try_backtrack()
        }
    }

    pub fn try_generate_complete_candidate(&mut self) -> bool {
        while !self.is_complete() {
            while self.is_current_cell_missing() {
                if !self.try_advance_source() {
                    return false;
                }
            }
            if !self.try_advance_resource() {
                return false;
            }
        }
        true
    }
}
