use std::path::PathBuf;

use super::L10nRegistry;
use crate::fluent::FluentBundle;

use unic_langid::LanguageIdentifier;

impl L10nRegistry {
    pub fn generate_bundles_for_lang_sync<'l>(
        &'l self,
        langid: &'l LanguageIdentifier,
        res_ids: &'l [PathBuf],
    ) -> impl Iterator<Item = FluentBundle> + 'l {
        self.generate_source_permutations(langid, res_ids)
            .map(move |sources| sources.into_iter().zip(res_ids))
            .filter_map(move |sources| {
                let mut bundle = FluentBundle::new(&[langid.clone()]);
                for (source, res_id) in sources {
                    if let Some(res) = source.fetch_file_sync(&langid, res_id) {
                        bundle.add_resource(res).unwrap();
                    } else {
                        return None;
                    }
                }
                Some(bundle)
            })
    }

    pub fn generate_bundles_sync<'l>(
        &'l self,
        lang_ids: &'l [LanguageIdentifier],
        res_ids: &'l [PathBuf],
    ) -> impl Iterator<Item = FluentBundle> + 'l {
        lang_ids
            .iter()
            .map(move |langid| self.generate_bundles_for_lang_sync(langid, res_ids))
            .flatten()
    }
}