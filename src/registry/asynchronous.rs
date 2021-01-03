use std::{
    pin::Pin,
    task::{Context, Poll},
};

use super::{L10nRegistry, L10nRegistryLocked};
use crate::solver::{AsyncTester, ParallelProblemSolver};
use crate::{
    fluent::{FluentBundle, FluentResource, FluentError},
    source::{ResourceOption, ResourceStatus},
};

use futures::{
    stream::{Collect, FuturesOrdered},
    Stream, StreamExt,
};
use std::future::Future;
use unic_langid::LanguageIdentifier;
use fluent_fallback::generator::BundleStream;
use std::rc::Rc;

impl<'a> L10nRegistryLocked<'a> {}

impl L10nRegistry {
    pub fn generate_bundles_for_lang(
        &self,
        langid: LanguageIdentifier,
        resource_ids: Vec<String>,
    ) -> GenerateBundles {
        let lang_ids = vec![langid];

        GenerateBundles::new(self.clone(), lang_ids, resource_ids)
    }

    pub fn generate_bundles(
        &self,
        lang_ids: Vec<LanguageIdentifier>,
        resource_ids: Vec<String>,
    ) -> GenerateBundles {
        GenerateBundles::new(self.clone(), lang_ids, resource_ids)
    }
}

enum State {
    Empty,
    Locale(LanguageIdentifier),
    Solver {
        locale: LanguageIdentifier,
        solver: ParallelProblemSolver<GenerateBundles>,
    },
}

impl Default for State {
    fn default() -> Self {
        Self::Empty
    }
}

impl State {
    fn get_locale(&self) -> &LanguageIdentifier {
        match self {
            Self::Locale(locale) => locale,
            Self::Solver { locale, .. } => locale,
            Self::Empty => unreachable!(),
        }
    }

    fn take_solver(&mut self) -> ParallelProblemSolver<GenerateBundles> {
        replace_with::replace_with_or_default_and_return(self, |self_| match self_ {
            Self::Solver { locale, solver } => (solver, Self::Locale(locale)),
            _ => unreachable!(),
        })
    }

    fn put_back_solver(&mut self, solver: ParallelProblemSolver<GenerateBundles>) {
        replace_with::replace_with_or_default(self, |self_| match self_ {
            Self::Locale(locale) => Self::Solver { locale, solver },
            _ => unreachable!(),
        })
    }
}

pub struct GenerateBundles {
    reg: L10nRegistry,
    locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    res_ids: Vec<String>,
    state: State,
}

impl GenerateBundles {
    fn new(reg: L10nRegistry, locales: Vec<LanguageIdentifier>, res_ids: Vec<String>) -> Self {
        Self {
            reg,
            locales: locales.into_iter(),
            res_ids,
            state: State::Empty,
        }
    }
}

pub type ResourceSetStream = Collect<FuturesOrdered<ResourceStatus>, Vec<ResourceOption>>;
pub struct TestResult(ResourceSetStream);
impl std::marker::Unpin for TestResult {}

impl Future for TestResult {
    type Output = Vec<bool>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pinned = Pin::new(&mut self.0);
        pinned
            .poll(cx)
            .map(|set| set.iter().map(|c| c.is_some()).collect())
    }
}

impl<'l> AsyncTester for GenerateBundles {
    type Result = TestResult;

    fn test_async(&self, query: Vec<(usize, usize)>) -> Self::Result {
        let locale = self.state.get_locale();
        let lock = self.reg.lock();

        let stream = query
            .iter()
            .map(|(res_idx, source_idx)| {
                let res = &self.res_ids[*res_idx];
                lock.source_idx(*source_idx).fetch_file(locale, res)
            })
            .collect::<FuturesOrdered<_>>();
        TestResult(stream.collect())
    }
}

impl BundleStream for GenerateBundles {
    type Resource = Rc<FluentResource>;
}

impl Stream for GenerateBundles {
    type Item = Result<FluentBundle, (FluentBundle, Vec<FluentError>)>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let State::Solver { .. } = self.state {
                let mut solver = self.state.take_solver();
                let pinned_solver = Pin::new(&mut solver);
                match pinned_solver.poll_next(cx, &self) {
                    std::task::Poll::Ready(order) => {
                        if let Some(order) = order {
                            let locale = self.state.get_locale();
                            let bundle = self.reg.lock().bundle_from_order(
                                locale.clone(),
                                &order,
                                &self.res_ids,
                            );
                            self.state.put_back_solver(solver);
                            if bundle.is_some() {
                                return bundle.into();
                            } else {
                                continue;
                            }
                        } else {
                            self.state = State::Empty;
                            continue;
                        }
                    }
                    std::task::Poll::Pending => {
                        self.state.put_back_solver(solver);
                    }
                }
            } else if let Some(locale) = self.locales.next() {
                let solver = ParallelProblemSolver::new(self.res_ids.len(), self.reg.lock().len());
                self.state = State::Solver { locale, solver };
            } else {
                return None.into();
            }
        }
    }
}
