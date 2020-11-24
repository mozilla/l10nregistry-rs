#[derive(Debug)]
pub struct Solution {
    pub width: usize,
    pub depth: usize,
    pub candidate: Vec<usize>,
    pub idx: usize,
    pub dirty: bool,
}

impl Solution {
    #[inline]
    pub fn try_next_source(&mut self) -> bool {
        if self.candidate[self.idx] < self.depth - 1 {
            self.candidate[self.idx] += 1;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn try_next_resource(&mut self) -> bool {
        if self.idx < self.width - 1 {
            self.idx += 1;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn try_backtrack(&mut self) -> bool {
        while self.candidate[self.idx] == self.depth - 1 {
            if self.idx == 0 {
                return false;
            }
            self.idx -= 1;
        }
        self.candidate[self.idx] += 1;
        self.prune();
        true
    }

    #[inline]
    pub fn prune(&mut self) {
        for i in self.idx + 1..self.width {
            self.candidate[i] = 0;
        }
    }

    #[inline]
    pub fn is_complete(&self) -> bool {
        self.idx == self.width - 1
    }

    #[inline]
    pub fn advance(&mut self) -> bool {
        if self.try_next_resource() {
            true
        } else {
            self.bail()
        }
    }

    #[inline]
    pub fn advance_to_completion(&mut self) -> bool {
        while !self.is_complete() {
            if !self.try_next_resource() {
                if !self.bail() {
                    return false;
                }
            }
        }
        true
    }

    #[inline]
    pub fn bail(&mut self) -> bool {
        if self.try_next_source() {
            true
        } else {
            self.try_backtrack()
        }
    }
}
