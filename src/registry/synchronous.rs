use std::path::Path;

use super::L10nRegistry;
use crate::fluent::FluentBundle;

use unic_langid::LanguageIdentifier;

impl L10nRegistry {
    pub fn generate_bundles_for_lang_sync<'l, P>(
        &'l self,
        langid: &'l LanguageIdentifier,
        res_ids: impl IntoIterator<Item = P> + Clone + 'l,
    ) -> impl Iterator<Item = FluentBundle> + 'l
    where
        P: AsRef<Path> + Clone + 'l,
    {
        self.generate_source_permutations(langid, res_ids.clone())
            .map(move |sources| sources.into_iter().zip(res_ids.clone()))
            .filter_map(move |sources| {
                let mut bundle = FluentBundle::new(&[langid.clone()]);
                for (source, res_id) in sources {
                    if let Some(res) = source.fetch_file_sync(&langid, res_id.as_ref()) {
                        bundle.add_resource(res).unwrap();
                    } else {
                        return None;
                    }
                }
                Some(bundle)
            })
    }

    pub fn generate_bundles_sync<'l, P>(
        &'l self,
        lang_ids: impl IntoIterator<Item = &'l LanguageIdentifier> + 'l,
        res_ids: impl IntoIterator<Item = P> + Clone + 'l,
    ) -> impl Iterator<Item = FluentBundle> + 'l
    where
        P: AsRef<Path> + Clone + 'l,
    {
        lang_ids
            .into_iter()
            .map(move |langid| self.generate_bundles_for_lang_sync(langid, res_ids.clone()))
            .flatten()
    }
}
