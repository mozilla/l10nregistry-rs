use crate::registry::L10nRegistry;
use crate::solver::{SerialProblemSolver, SyncTester};
use fluent_bundle::{FluentBundle, FluentResource};
use fluent_fallback::generator::BundleIterator;
use std::cell::RefCell;
use std::rc::Rc;
use unic_langid::LanguageIdentifier;

pub struct GenerateBundlesSync<'l> {
    reg: &'l L10nRegistry,
    locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    res_ids: Vec<String>,
    solver: Option<(LanguageIdentifier, RefCell<SerialProblemSolver>)>,
    adapt_bundle: Option<fn(&mut FluentBundle<Rc<FluentResource>>)>,
}

impl<'l> GenerateBundlesSync<'l> {
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

impl<'l> SyncTester for GenerateBundlesSync<'l> {
    fn test_sync(&self, res_idx: usize, source_idx: usize) -> bool {
        let locale = &self.solver.as_ref().unwrap().0;
        let res = &self.res_ids[res_idx];
        self.reg.sources[source_idx]
            .fetch_file_sync(locale, res)
            .is_some()
    }
}

impl<'l> BundleIterator for GenerateBundlesSync<'l> {
    type Resource = Rc<FluentResource>;

    fn prefetch(&mut self) {}
}

impl<'l> Iterator for GenerateBundlesSync<'l> {
    type Item = FluentBundle<Rc<FluentResource>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((locale, solver)) = &self.solver {
                // I could also take and then set the solver here
                if let Some(order) = solver.borrow_mut().next(self) {
                    return Some(self.bundle_from_order(locale.clone(), order).unwrap());
                }
                self.solver = None;
            }

            let locale = self.locales.next()?;
            let solver = SerialProblemSolver::new(self.res_ids.len(), self.reg.sources.len());
            self.solver = Some((locale, RefCell::new(solver)));
        }
    }
}
