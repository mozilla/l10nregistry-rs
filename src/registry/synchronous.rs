use std::time::Instant;
use std::{iter::Rev, ops::Range, rc::Rc};

use super::{L10nRegistry, L10nRegistryLocked};
use crate::fluent::{FluentBundle, FluentResource};

use unic_langid::LanguageIdentifier;

pub type FluentResourceSet = Vec<Rc<FluentResource>>;

impl<'a> L10nRegistryLocked<'a> {
    pub(crate) fn generate_resource_set_sync<P>(
        &self,
        langid: &LanguageIdentifier,
        source_order: &[usize],
        resource_ids: &[P],
    ) -> Option<FluentResourceSet>
    where
        P: AsRef<str>,
    {
        debug_assert_eq!(source_order.len(), resource_ids.len());
        let mut result = vec![];
        for (&idx, path) in source_order
            .iter()
            .zip(resource_ids.iter().map(AsRef::as_ref))
        {
            let source = self.source_idx(idx);
            if let Some(resource) = source.fetch_file_sync(langid, path) {
                result.push(resource)
            } else {
                return None;
            }
        }
        Some(result)
    }

    pub fn get_file_from_source(
        &self,
        langid: &LanguageIdentifier,
        source: usize,
        res_id: &str,
    ) -> Option<Rc<FluentResource>> {
        let source = self.source_idx(source);
        if let Some(resource) = source.fetch_file_sync(langid, res_id) {
            Some(resource)
        } else {
            None
        }
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

// use crate::solver::{ProblemSolver, SerialProblemSolver};

pub struct GenerateBundlesSync {
    // iter: ProblemSolver,
    reg: L10nRegistry,
    lang_ids: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    resource_ids: Vec<String>,
    state: Option<(
        LanguageIdentifier,
        itertools::MultiProduct<Rev<Range<usize>>>,
    )>,
}

impl GenerateBundlesSync {
    fn new(
        reg: L10nRegistry,
        lang_ids: Vec<LanguageIdentifier>,
        resource_ids: Vec<String>,
    ) -> Self {
        Self {
            reg,
            lang_ids: lang_ids.into_iter(),
            resource_ids,
            state: None,
            // iter: ProblemSolver::new(resource_ids.clone(), lang_ids[0].clone(), reg),
        }
    }
}

impl Iterator for GenerateBundlesSync {
    type Item = FluentBundle;

    fn next(&mut self) -> Option<Self::Item> {
        let now = Instant::now();
        // println!("GenerateBundlesSync::next");
        loop {
            // if let Some(bundle) = self.iter.next_bundle() {
            //     let diff = now.elapsed().as_nanos();
            //     println!("GenerateBundlesSync::next end: {} ns.", diff);
            //     return Some(bundle);
            // } else {
            //     return None;
            // }
            if let Some((ref mut langid, ref mut source_orders)) = self.state {
                for source_order in source_orders {
                    // println!("GenerateBundlesSync::next source_order: {:#?}.", source_order);
                    if let Some(set) = self.reg.lock().generate_resource_set_sync(
                        langid,
                        &source_order,
                        &self.resource_ids,
                    ) {
                        let mut bundle = FluentBundle::new(&[langid.clone()]);
                        for res in set {
                            bundle.add_resource(res).unwrap()
                        }
                        let diff = now.elapsed().as_nanos();
                        println!("GenerateBundlesSync::next end: {} ns.", diff);
                        return Some(bundle);
                    }
                }
            }

            let lang_id = self.lang_ids.next()?;
            let source_orders = super::permute_iter(self.reg.lock().len(), self.resource_ids.len());
            self.state = Some((lang_id, source_orders))
        }
    }
}
