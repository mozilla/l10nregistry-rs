use std::rc::Rc;

use super::{L10nRegistry, L10nRegistryLocked};
use crate::fluent::{FluentBundle, FluentResource};

use unic_langid::LanguageIdentifier;

impl<'a> L10nRegistryLocked<'a> {
    pub fn get_file_from_source(
        &self,
        langid: &LanguageIdentifier,
        source: usize,
        res_id: &str,
    ) -> Option<Rc<FluentResource>> {
        self.source_idx(source).fetch_file_sync(langid, res_id)
    }
}

impl L10nRegistry {
    pub fn generate_bundles_for_lang_sync(
        &self,
        langid: LanguageIdentifier,
        resource_ids: Vec<String>,
    ) -> GenerateBundlesSync {
        let lang_ids = vec![langid];

        GenerateBundlesSync::new(self.clone(), lang_ids, resource_ids)
    }

    pub fn generate_bundles_sync(
        &self,
        lang_ids: Vec<LanguageIdentifier>,
        resource_ids: Vec<String>,
    ) -> GenerateBundlesSync {
        GenerateBundlesSync::new(self.clone(), lang_ids, resource_ids)
    }
}

use crate::solver::SerialProblemSolver;

pub struct GenerateBundlesSync {
    solver: Option<SerialProblemSolver>,
    resource_ids: Vec<String>,
    reg: L10nRegistry,
    lang_ids: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
}

impl GenerateBundlesSync {
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

impl Iterator for GenerateBundlesSync {
    type Item = FluentBundle;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(solver) = &mut self.solver {
                if let Some(bundle) = solver.next_bundle() {
                    return Some(bundle);
                } else {
                    self.solver = None;
                    continue;
                }
            } else if let Some(lang) = self.lang_ids.next() {
                let solver =
                    SerialProblemSolver::new(self.resource_ids.clone(), lang, self.reg.clone());
                self.solver = Some(solver);
                continue;
            } else {
                return None;
            }
        }
    }
}
