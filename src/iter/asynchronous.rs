use futures::{ready, FutureExt, Stream};
use std::pin::Pin;
use tracing::debug;

pub trait Tester {
    fn test_cell(&self, resource: usize, source: usize) -> Option<bool>;
    fn get_cell(
        &mut self,
        resource: usize,
        source: usize,
    ) -> Pin<Box<dyn std::future::Future<Output = bool>>>;
}

pub struct AsyncSourcesGenerator<'g, T> {
    pub sources: Vec<usize>,
    pub resources: Vec<usize>,
    // We could make it Vec<Option<usize>> and switch bad cells to None
    pub indexes: Vec<usize>,
    pub res_idx: usize,
    pub done: bool,
    pub tester: &'g mut T,
}

impl<'g, T> AsyncSourcesGenerator<'g, T> {
    pub fn new(resources: Vec<usize>, sources: Vec<usize>, tester: &'g mut T) -> Self {
        let mut indexes = vec![];
        indexes.resize_with(resources.len(), || 0);
        let res_idx = 0;
        Self {
            sources,
            resources,
            indexes,
            res_idx,
            done: false,
            tester,
        }
    }

    pub fn test_cell(&self) -> Option<bool>
    where
        T: Tester,
    {
        self.tester
            .test_cell(self.res_idx, self.indexes[self.res_idx])
    }

    pub fn fetch_cell(&mut self) -> Pin<Box<dyn std::future::Future<Output = bool>>>
    where
        T: Tester,
    {
        self.tester
            .get_cell(self.res_idx, self.indexes[self.res_idx])
    }

    pub fn bail(&mut self) -> bool {
        if self.source_idx() < self.sources.len() - 1 {
            self.indexes[self.res_idx] += 1;
            true
        } else {
            loop {
                if self.res_idx > 0 {
                    self.indexes[self.res_idx] = 0;
                    self.res_idx -= 1;
                    if self.source_idx() < self.sources.len() - 1 {
                        self.indexes[self.res_idx] += 1;
                        return true;
                    }
                } else {
                    return false;
                }
            }
        }
    }

    pub fn bump_cell(&mut self) -> bool {
        if self.res_idx < self.resources.len() - 1 {
            self.res_idx += 1;
            true
        } else {
            loop {
                if self.source_idx() < self.sources.len() - 1 {
                    self.indexes[self.res_idx] += 1;
                    return true;
                } else if self.res_idx > 0 {
                    self.indexes[self.res_idx] = 0;
                    self.res_idx -= 1;
                } else {
                    self.done = true;
                    return false;
                }
            }
        }
    }

    pub fn last_col(&self) -> bool {
        let result = self.res_idx == self.resources.len() - 1;
        debug!("last_col: {}", result);
        result
    }

    pub fn source_idx(&self) -> usize {
        self.indexes[self.res_idx]
    }
}

impl<'g, T> Stream for AsyncSourcesGenerator<'g, T>
where
    T: Tester,
{
    type Item = Vec<usize>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            if self.done {
                return None.into();
            }

            // if self.test_cell() == Some(false) {
            //     if !self.bump_cell() {
            //         return None.into();
            //     } else {
            //         continue;
            //     }
            // }

            // if !self.fetch_cell() {
            //     if !self.bail() {
            //         return None.into();
            //     } else {
            //         continue;
            //     }
            // }

            // self.indexes[self.res_idx] = self.source_idx();

            // if self.last_col() {
            //     let result = self.indexes.clone();
            //     self.indexes.resize_with(self.resources.len(), || 0);
            //     self.bump_cell();
            //     return Some(result).into();
            // }

            if !self.bump_cell() {
                return None.into();
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "tokio")]
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

    #[tokio::test]
    async fn test_async() {
        // let tests = &[
        //     (
        //         vec![0, 1, 2],
        //         vec![0, 1],
        //         vec![(1, 0), (2, 1)],
        //         vec![vec![0, 1, 0], vec![1, 1, 0]],
        //     ),
        //     (
        //         vec![0, 1],
        //         vec![0, 1],
        //         vec![(0, 1), (1, 0)],
        //         vec![vec![0, 1]],
        //     ),
        //     (
        //         vec![0, 1, 2, 3],
        //         vec![0, 1],
        //         vec![(0, 0), (0, 1)],
        //         vec![],
        //     ),
        //     (
        //         vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        //         vec![0, 1, 2],
        //         vec![(0, 1), (1, 0)],
        //         vec![vec![0, 1]],
        //     ),
        // ];

        // for (resources, sources, missing, expected) in tests {
        //     let mut fs = MockFileSource::new(missing.to_vec());

        //     let gen = AsyncSourcesGenerator::new(resources.clone(), sources.clone(), &mut fs);
        //     let mut count = 0;
        //     for (i, val) in gen.into_iter().enumerate() {
        //         assert!(expected.len() > i);
        //         assert_eq!(&val, expected.get(i).unwrap());
        //         count += 1;
        //     }
        //     assert_eq!(count, expected.len());
        // }
    }
}
