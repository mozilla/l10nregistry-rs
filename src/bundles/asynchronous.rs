use crate::registry::L10nRegistry;
use crate::solver::{AsyncTester, ParallelProblemSolver};
use crate::source::ResourceStatus;
use fluent_bundle::{FluentBundle, FluentResource};
use fluent_fallback::generator::BundleStream;
use futures::future::join_all;
use futures::Stream;
use std::future::Future;
use std::rc::Rc;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use unic_langid::LanguageIdentifier;

pub struct GenerateBundles<'l> {
    reg: &'l L10nRegistry,
    locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    res_ids: Vec<String>,
    solver: Option<(LanguageIdentifier, ParallelProblemSolver<Self>)>,
    adapt_bundle: Option<fn(&mut FluentBundle<Rc<FluentResource>>)>,
}

impl<'l> GenerateBundles<'l> {
    pub fn new(
        reg: &'l L10nRegistry,
        locales: Vec<LanguageIdentifier>,
        res_ids: Vec<String>,
        adapt_bundle: Option<fn(&mut FluentBundle<Rc<FluentResource>>)>,
    ) -> Self {
        Self {
            locales: locales.into_iter(),
            res_ids,
            reg,
            solver: None,
            adapt_bundle,
        }
    }

    fn bundle_from_order(
        &self,
        locale: LanguageIdentifier,
        order: &[usize],
    ) -> Option<FluentBundle<Rc<FluentResource>>> {
        let mut bundle = FluentBundle::new(vec![locale.clone()]);
        for (res_idx, source_idx) in order.iter().enumerate() {
            let res = self.reg.sources[*source_idx]
                .fetch_file_sync(&locale, &self.res_ids[res_idx])
                .unwrap();
            bundle.add_resource(res).unwrap();
        }
        if let Some(adapt) = self.adapt_bundle {
            adapt(&mut bundle);
        }
        Some(bundle)
    }
}

pub struct TestResult(futures::future::JoinAll<ResourceStatus>);

impl std::marker::Unpin for TestResult {}

impl Future for TestResult {
    type Output = Vec<bool>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pinned = Pin::new(&mut self.0);
        pinned.poll(cx)
    }
}

impl<'l> AsyncTester for GenerateBundles<'l> {
    type Result = TestResult;

    fn test_async(&self, query: Vec<(usize, usize)>) -> Self::Result {
        let locale = &self.solver.as_ref().unwrap().0;

        let futures = query
            .into_iter()
            .map(|(res_idx, source_idx)| {
                let res = &self.res_ids[res_idx];
                self.reg.sources[source_idx].fetch_file(locale, res)
            })
            .collect::<Vec<_>>();
        TestResult(join_all(futures))
    }
}

impl<'l> BundleStream for GenerateBundles<'l> {
    type Resource = Rc<FluentResource>;

    fn prefetch(&mut self) {}
}

impl<'l> Stream for GenerateBundles<'l> {
    type Item = FluentBundle<Rc<FluentResource>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some((locale, mut solver)) = self.solver.take() {
                let pinned_solver = Pin::new(&mut solver);
                //XXX: During this test locale is empty so test_async panics
                match pinned_solver.poll_next(cx, &self) {
                    std::task::Poll::Ready(order) => {
                        if let Some(order) = order {
                            return Some(self.bundle_from_order(locale.clone(), &order).unwrap())
                                .into();
                        } else {
                            self.solver = None;
                            continue;
                        }
                    }
                    std::task::Poll::Pending => {
                        self.solver = Some((locale, solver));
                    }
                }
            } else if let Some(locale) = self.locales.next() {
                let solver = ParallelProblemSolver::new(self.res_ids.len(), self.reg.sources.len());
                self.solver = Some((locale, solver));
            } else {
                return None.into();
            }
        }
    }
}
