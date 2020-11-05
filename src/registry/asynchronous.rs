use std::{
    pin::Pin,
    task::{Context, Poll},
};
use std::rc::Rc;

use super::L10nRegistry;
use crate::fluent::{FluentBundle, FluentResource};

use crate::solver::ParallelProblemSolver;
use futures::{ready, Stream};
use unic_langid::LanguageIdentifier;
use fluent_fallback::generator::BundleStream;

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

pub struct GenerateBundles {
    solver: Option<ParallelProblemSolver>,
    resource_ids: Vec<String>,
    reg: L10nRegistry,
    lang_ids: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
}

impl GenerateBundles {
    fn new(
        reg: L10nRegistry,
        lang_ids: Vec<LanguageIdentifier>,
        resource_ids: Vec<String>,
    ) -> Self {
        Self {
            lang_ids: lang_ids.into_iter(),
            resource_ids,
            reg,
            solver: None,
        }
    }
}

impl BundleStream for GenerateBundles {
    type Resource = Rc<FluentResource>;
}

impl Stream for GenerateBundles {
    type Item = FluentBundle;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(solver) = &mut self.solver {
                let solver = Pin::new(solver);
                if let Some(bundle) = ready!(solver.poll_next(cx)) {
                    return Some(bundle).into();
                } else {
                    self.solver = None;
                    continue;
                }
            } else if let Some(lang) = self.lang_ids.next() {
                let solver =
                    ParallelProblemSolver::new(self.resource_ids.clone(), lang, self.reg.clone());
                self.solver = Some(solver);
            } else {
                return None.into();
            }
        }
    }
}
