use super::ProblemSolver;
use crate::fluent::FluentBundle;
use crate::registry::asynchronous::ResourceSetStream;
use crate::registry::L10nRegistry;
use futures::ready;
use futures::FutureExt;
use futures::Stream;
use std::ops::{Deref, DerefMut};
use unic_langid::LanguageIdentifier;

pub struct ParallelProblemSolver {
    solver: ProblemSolver,
    current_stream: Option<ResourceSetStream>,
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

    async fn test_candidate(&mut self) -> Option<usize> {
        // println!("test_candidate: {:?}", self.solution.candidate);
        // for res_idx in 0..self.solution.width {
        //     let source_idx = self.solution.candidate[res_idx];
        //     if let Some(cell) = &self.solution.get_cell(res_idx, source_idx) {
        //         if cell.is_none() {
        //             return Some(res_idx);
        //         }
        //     } else {
        //         missing_idx.push(res_idx);
        //         request.push((&self.keys[res_idx], source_idx));
        //     }
        // }
        //
        // XXX: Make it a Result
        let request = self.solution.candidate.iter().enumerate();
        // .map(|(res_idx, source_idx)| {
        //     (res_idx, source_idx)
        // });

        // println!("test_complete_solution {:?}, {:?}", missing_resources, missing_sources);
        let set = self
            .reg
            .lock()
            .generate_resource_set(&self.langid, &self.keys, request)
            .await;
        // println!("test_complete_solution result {:?}", set);

        let mut first_fail = None;
        for (idx, res) in set.into_iter().enumerate() {
            let res_idx = missing_idx[idx];

            if first_fail.is_none() && res.is_none() {
                first_fail = Some(res_idx);
            }
            let source_idx = self.solution.candidate[res_idx];
            self.solution.cache[res_idx][source_idx] = Some(res);
        }
        first_fail
    }

    pub async fn next(&mut self) -> Option<&Vec<usize>> {
        if self.solution.dirty {
            if !self.solution.bail() {
                return None;
            }
            self.solution.dirty = false;
        }
        while self.solution.try_generate_complete_candidate() {
            if let Some(idx) = self.test_candidate().await {
                // println!("First error: {}", idx);
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

    pub async fn next_bundle(&mut self) -> Option<FluentBundle> {
        panic!()
    }
}

impl Stream for ParallelProblemSolver {
    type Item = FluentBundle;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        // 'outer: loop {
        //     if let Some(stream) = &mut self.current_stream {
        //         let set = ready!(stream.poll_unpin(cx));

        //             Ok(set) => {
        //                 self.current_stream = None;
        //                 let mut bundle = FluentBundle::new(&[self.langid.clone()]);
        //                 for res in set {
        //                     bundle.add_resource(res).unwrap()
        //                 }
        //                 self.solution.dirty = true;
        //                 return Some(bundle).into();
        //             }
        //             Err(idx) => {
        //                 self.solution.idx = idx;
        //                 self.solution.prune();
        //                 if !self.solution.bail() {
        //                     return None.into();
        //                 }
        //                 self.current_stream = None;
        //                 continue 'outer;
        //             }
        //         }
        //     } else {
        //         if self.solution.dirty {
        //             if !self.solution.bail() {
        //                 return None.into();
        //             }
        //             self.solution.dirty = false;
        //         }
        //         while self.solution.advance_to_completion() {
        //             self.current_stream = Some(self.test_complete_solution());
        //             continue 'outer;
        //         }
        //     }
        // }
        None.into()
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
