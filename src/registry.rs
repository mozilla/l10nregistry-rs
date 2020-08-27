use std::path::Path;

use itertools::Itertools;
use unic_langid::LanguageIdentifier;

use crate::fluent::FluentBundle;
use crate::source::FileSource;

#[derive(Default)]
pub struct L10nRegistry {
    pub sources: Vec<FileSource>,
}

impl L10nRegistry {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn register_sources(&mut self, sources: Vec<FileSource>) -> Result<(), ()> {
        for source in sources {
            if self.sources.contains(&source) {
                return Err(());
            }
            self.sources.push(source);
        }
        Ok(())
    }

    pub fn get_source(&self, name: &str) -> Option<&FileSource> {
        self.sources.iter().find(|source| source.name == name)
    }

    pub fn generate_sources_for_file<'l>(
        &'l self,
        langid: &'l LanguageIdentifier,
        res_id: &'l Path,
    ) -> impl Iterator<Item = &'l FileSource> + Clone {
        self.sources
            .iter()
            .filter(move |source| source.has_file(langid, res_id) != Some(false))
    }

    pub fn generate_source_permutations<'l>(
        &'l self,
        langid: &'l LanguageIdentifier,
        res_ids: &[&'l Path],
    ) -> impl Iterator<Item = Vec<&FileSource>> + 'l {
        res_ids
            .iter()
            .map(|res_id| self.generate_sources_for_file(langid, res_id))
            .multi_cartesian_product()
    }

    pub fn generate_bundles_for_lang_sync<'l>(
        &'l self,
        langid: &'l LanguageIdentifier,
        res_ids: &'l [&'l Path],
    ) -> impl Iterator<Item = FluentBundle> + 'l {
        self.generate_source_permutations(langid, res_ids)
            .map(move |sources| sources.into_iter().zip(res_ids))
            .filter_map(move |sources| {
                let mut bundle = FluentBundle { resources: vec![] };
                for (source, res_id) in sources {
                    if let Some(res) = source.fetch_file_sync(&langid, res_id) {
                        bundle.resources.push(res);
                    } else {
                        return None;
                    }
                }
                Some(bundle)
            })
    }

    pub fn generate_bundles_sync<'l>(
        &'l self,
        lang_ids: &'l [&'l LanguageIdentifier],
        res_ids: &'l [&'l Path],
    ) -> impl Iterator<Item = FluentBundle> + 'l {
        lang_ids
            .iter()
            .map(move |langid| self.generate_bundles_for_lang_sync(langid, res_ids))
            .flatten()
    }
}
