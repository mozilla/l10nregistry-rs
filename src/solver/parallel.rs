use super::ProblemSolver;
use crate::registry::L10nRegistry;
use std::ops::{Deref, DerefMut};
use unic_langid::LanguageIdentifier;

use crate::{
    fluent::FluentBundle,
    source::{RcResourceOption, ResourceStatus},
};
use futures::{
    ready,
    stream::{Collect, FuturesOrdered},
    FutureExt, Stream, StreamExt,
};

pub type ResourceSetStream = Collect<FuturesOrdered<ResourceStatus>, Vec<RcResourceOption>>;

pub struct ParallelProblemSolver {
    solver: ProblemSolver,
    current_stream: Option<(ResourceSetStream, Vec<usize>)>,
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
            current_stream: None,
        }
    }

    fn try_generate_test_stream(&mut self) -> Result<(ResourceSetStream, Vec<usize>), usize> {
        let mut testing_cells = vec![];
        let stream = {
            let lock = self.reg.lock();
            self.solution
                .candidate
                .iter()
                .enumerate()
                .filter_map(|(res_idx, &source_idx)| {
                    match self.solution.cache[res_idx][source_idx] {
                        None => {
                            let source = lock.source_idx(source_idx);
                            let path = self.keys[res_idx].as_str();
                            let langid = &self.langid;
                            testing_cells.push(res_idx);
                            Some(Ok(source.fetch_file(langid, path)))
                        }
                        Some(None) => Some(Err(res_idx)),
                        _ => None,
                    }
                })
                .collect::<Result<FuturesOrdered<_>, usize>>()
        };
        match stream {
            Ok(stream) => Ok((stream.collect(), testing_cells)),
            Err(idx) => Err(idx),
        }
    }

    async fn test_candidate(&mut self) -> Result<(), usize> {
        let (stream, testing_cells) = self.try_generate_test_stream()?;
        self.apply_test_result(stream.await, &testing_cells)
    }

    fn apply_test_result(
        &mut self,
        resources: Vec<RcResourceOption>,
        testing_cells: &[usize],
    ) -> Result<(), usize> {
        for (missing_idx, res) in resources.iter().enumerate() {
            let res_idx = testing_cells[missing_idx];
            if let Some(res) = res {
                let source_idx = self.solution.candidate[res_idx];
                self.solution.cache[res_idx][source_idx] = Some(Some(res.clone()));
            } else {
                return Err(res_idx);
            }
        }
        Ok(())
    }

    pub async fn next(&mut self) -> Option<&Vec<usize>> {
        if self.solution.depth == 0 || self.solution.width == 0 {
            return None;
        }

        if self.solution.dirty {
            if !self.solution.bail() {
                return None;
            }
            self.solution.dirty = false;
        }
        while self.solution.try_generate_complete_candidate() {
            if let Err(idx) = self.test_candidate().await {
                self.solution.idx = idx;
                if !self.solution.prune() {
                    return None;
                }
                if !self.solution.bail() {
                    return None;
                }
                continue;
            } else {
                self.solution.dirty = true;
                return Some(&self.solution.candidate);
            }
        }
        None
    }
}

impl Stream for ParallelProblemSolver {
    type Item = FluentBundle;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.solution.depth == 0 || self.solution.width == 0 {
            return None.into();
        }

        'outer: loop {
            if let Some((stream, testing_cells)) = &mut self.current_stream {
                let set = ready!(stream.poll_unpin(cx));
                let testing_cells = testing_cells.clone();

                if let Err(idx) = self.apply_test_result(set, &testing_cells) {
                    self.solution.idx = idx;
                    self.solution.prune();
                    if !self.solution.bail() {
                        return None.into();
                    }
                    self.current_stream = None;
                    continue 'outer;
                } else {
                    let bundle = self.get_bundle();
                    self.current_stream = None;
                    self.solution.dirty = true;
                    return Some(bundle).into();
                }
            } else {
                if self.solution.dirty {
                    if !self.solution.bail() {
                        return None.into();
                    }
                    self.solution.dirty = false;
                }
                while self.solution.try_generate_complete_candidate() {
                    match self.try_generate_test_stream() {
                        Ok((stream, testing_cells)) => {
                            self.current_stream = Some((stream, testing_cells));
                            continue 'outer;
                        }
                        Err(idx) => {
                            self.solution.idx = idx;
                            self.solution.prune();
                            if !self.solution.bail() {
                                return None.into();
                            }
                        }
                    }
                }
                return None.into();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_async() {
        use super::*;
        use crate::solver::testing::get_scenarios;
        use unic_langid::LanguageIdentifier;

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
