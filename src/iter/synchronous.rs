use tracing::debug;

// Streaming Iterator
// BSF approach with backtracking
// No memoization yet
pub trait Tester {
    fn test_cell(&self, resource: usize, source: usize) -> Option<bool>;
    fn get_cell(&mut self, resource: usize, source: usize) -> bool;
}

struct Solution {
    width: usize,
    depth: usize,
    orders: Vec<usize>,
    idx: usize,
}

impl Solution {
    #[inline]
    fn try_bump_cell(&mut self) -> bool {
        if self.orders[self.idx] < self.depth - 1 {
            self.orders[self.idx] += 1;
            true
        } else {
            false
        }
    }

    #[inline]
    fn try_advance(&mut self) -> bool {
        if self.idx < self.width - 1 {
            self.idx += 1;
            true
        } else {
            false
        }
    }

    #[inline]
    fn try_backtrack(&mut self) -> bool {
        while self.orders[self.idx] == self.depth - 1 {
            if self.idx == 0 {
                return false;
            }
            self.idx -= 1;
        }
        true
    }

    #[inline]
    fn is_complete(&self) -> bool {
        self.idx == self.width - 1
    }

    #[inline]
    fn current_source(&self) -> usize {
        self.orders[self.idx]
    }
}

// We could make it Vec<Vec<Option<usize>>> and store bad cells so that we don't
// have to re-test them after pruning

// 1. Solve for "one missing is okay" with test if solution exeedes tershold before bailing
// 2. Solve the too many permutations by clustering sources and testing langpack and packages
//    separately without crosspolination. (will it work for micro locales that want to fallback on
//    fuller locales?)
pub struct SyncSourcesGenerator<'g, T> {
    solution: Solution,

    pub done: bool,
    pub tester: &'g mut T,
}

impl<'g, T> SyncSourcesGenerator<'g, T> {
    pub fn new(resources: Vec<usize>, sources: Vec<usize>, tester: &'g mut T) -> Self {
        let width = resources.len();
        let orders = vec![0; width];
        let depth = sources.len();

        Self {
            solution: Solution {
                width,
                depth,
                orders,
                idx: 0,
            },

            done: false,
            tester,
        }
    }

    #[inline]
    fn advance(&mut self) -> bool {
        if self.solution.try_advance() {
            true
        } else {
            self.bail()
        }
    }

    #[inline]
    fn test_cell(&self) -> Option<bool>
    where
        T: Tester,
    {
        self.tester
            .test_cell(self.solution.idx, self.solution.current_source())
    }

    #[inline]
    fn fetch_cell(&mut self) -> bool
    where
        T: Tester,
    {
        self.tester
            .get_cell(self.solution.idx, self.solution.current_source())
    }

    #[inline]
    fn bail(&mut self) -> bool {
        if self.solution.try_bump_cell() {
            true
        } else {
            if !self.solution.try_backtrack() {
                self.done = true;
                return false;
            }

            self.solution.try_bump_cell();

            for i in self.solution.idx + 1..self.solution.width {
                self.solution.orders[i] = 0;
            }
            true
        }
    }

    #[inline]
    pub fn next(&mut self) -> Option<&Vec<usize>>
    where
        T: Tester,
    {
        if self.solution.is_complete() {
            self.bail();
        }
        while !self.done {
            if self.test_cell() == Some(false) || !self.fetch_cell() {
                self.bail();
                continue;
            }

            if self.solution.is_complete() {
                return Some(&self.solution.orders);
            } else {
                self.advance();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct MockFileSource {
        missing: Vec<(usize, usize)>,
    }

    impl MockFileSource {
        pub fn new(missing: Vec<(usize, usize)>) -> Self {
            Self { missing }
        }
    }

    impl Tester for MockFileSource {
        fn test_cell(&self, _resource: usize, _source: usize) -> Option<bool> {
            None
        }

        fn get_cell(&mut self, resource: usize, source: usize) -> bool {
            for cell in &self.missing {
                if cell == &(resource, source) {
                    return false;
                }
            }
            return true;
        }
    }

    #[test]
    fn test_sync() {
        let tests = &[
            (
                vec![0, 1, 2],
                vec![0, 1],
                vec![(1, 0), (2, 1)],
                vec![vec![0, 1, 0], vec![1, 1, 0]],
            ),
            // (
            //     vec![0, 1],
            //     vec![0, 1],
            //     vec![(0, 1), (1, 0)],
            //     vec![vec![0, 1]],
            // ),
            // (vec![0, 1, 2, 3], vec![0, 1], vec![(0, 0), (0, 1)], vec![]),
        ];

        for (resources, sources, missing, expected) in tests {
            let mut fs = MockFileSource::new(missing.to_vec());

            let mut gen = SyncSourcesGenerator::new(resources.clone(), sources.clone(), &mut fs);
            let mut i = 0;
            while let Some(order) = gen.next() {
                assert!(expected.len() > i);
                assert_eq!(order, expected.get(i).unwrap());
                i += 1;
            }
            assert_eq!(i, expected.len());
        }
    }
}
