mod asynchronous;
mod synchronous;

use std::path::Path;

use itertools::Itertools;
use unic_langid::LanguageIdentifier;

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
}
