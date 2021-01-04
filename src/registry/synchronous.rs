use super::{L10nRegistry, L10nRegistryLocked};
use crate::fluent::{FluentBundle, FluentError, FluentResource};
use crate::solver::{SerialProblemSolver, SyncTester};
use fluent_fallback::generator::BundleIterator;
use std::rc::Rc;

use unic_langid::LanguageIdentifier;

impl<'a> L10nRegistryLocked<'a> {
    pub(crate) fn bundle_from_order(
        &self,
        locale: LanguageIdentifier,
        source_order: &[usize],
        res_ids: &[String],
    ) -> Option<Result<FluentBundle, (FluentBundle, Vec<FluentError>)>> {
        let mut bundle = FluentBundle::new(vec![locale.clone()]);

        if let Some(adapt_bundle) = self.adapt_bundle {
            adapt_bundle(&mut bundle);
        }

        let mut errors = vec![];

        for (&source_idx, path) in source_order.iter().zip(res_ids.iter()) {
            let source = self.source_idx(source_idx);
            if let Some(res) = source.fetch_file_sync(&locale, path, false) {
                if let Err(err) = bundle.add_resource(res) {
                    errors.extend(err);
                }
            } else {
                return None;
            }
        }

        if errors.is_empty() {
            Some(Ok(bundle))
        } else {
            Some(Err((bundle, errors)))
        }
    }
}

impl<P> L10nRegistry<P>
where
    P: Clone,
{
    pub fn generate_bundles_for_lang_sync(
        &self,
        langid: LanguageIdentifier,
        resource_ids: Vec<String>,
    ) -> GenerateBundlesSync<P> {
        let lang_ids = vec![langid];

        GenerateBundlesSync::new(self.clone(), lang_ids, resource_ids)
    }

    pub fn generate_bundles_sync(
        &self,
        lang_ids: Vec<LanguageIdentifier>,
        resource_ids: Vec<String>,
    ) -> GenerateBundlesSync<P> {
        GenerateBundlesSync::new(self.clone(), lang_ids, resource_ids)
    }
}

enum State {
    Empty,
    Locale(LanguageIdentifier),
    Solver {
        locale: LanguageIdentifier,
        solver: SerialProblemSolver,
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

    fn take_solver(&mut self) -> SerialProblemSolver {
        replace_with::replace_with_or_default_and_return(self, |self_| match self_ {
            Self::Solver { locale, solver } => (solver, Self::Locale(locale)),
            _ => unreachable!(),
        })
    }

    fn put_back_solver(&mut self, solver: SerialProblemSolver) {
        replace_with::replace_with_or_default(self, |self_| match self_ {
            Self::Locale(locale) => Self::Solver { locale, solver },
            _ => unreachable!(),
        })
    }
}

pub struct GenerateBundlesSync<P> {
    reg: L10nRegistry<P>,
    locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    res_ids: Vec<String>,
    state: State,
}

impl<P> GenerateBundlesSync<P> {
    fn new(reg: L10nRegistry<P>, locales: Vec<LanguageIdentifier>, res_ids: Vec<String>) -> Self {
        Self {
            reg,
            locales: locales.into_iter(),
            res_ids,
            state: State::Empty,
        }
    }
}

impl<P> SyncTester for GenerateBundlesSync<P> {
    fn test_sync(&self, res_idx: usize, source_idx: usize) -> bool {
        let locale = self.state.get_locale();
        let res = &self.res_ids[res_idx];
        self.reg
            .lock()
            .source_idx(source_idx)
            .fetch_file_sync(locale, res, false)
            .is_some()
    }
}

impl<P> BundleIterator for GenerateBundlesSync<P> {
    type Resource = Rc<FluentResource>;
}

impl<P> Iterator for GenerateBundlesSync<P> {
    type Item = Result<FluentBundle, (FluentBundle, Vec<FluentError>)>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let State::Solver { .. } = self.state {
                let mut solver = self.state.take_solver();
                if let Some(order) = solver.next(self) {
                    let locale = self.state.get_locale();
                    let bundle =
                        self.reg
                            .lock()
                            .bundle_from_order(locale.clone(), order, &self.res_ids);
                    self.state.put_back_solver(solver);
                    if bundle.is_some() {
                        return bundle;
                    } else {
                        continue;
                    }
                }
                self.state = State::Empty;
            }

            let locale = self.locales.next()?;
            let solver = SerialProblemSolver::new(self.res_ids.len(), self.reg.lock().len());
            self.state = State::Solver { locale, solver };
        }
    }
}
