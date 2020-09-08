use futures::stream::Stream;
use futures::stream::{self, StreamExt};
use unic_langid::LanguageIdentifier;

use super::L10nRegistry;
use crate::fluent::FluentBundle;

impl L10nRegistry {
    pub fn generate_bundles_for_lang<'l, P>(
        &'l self,
        langid: &'l LanguageIdentifier,
        res_ids: impl IntoIterator<Item = P> + Clone + 'l,
    ) -> impl Stream<Item = FluentBundle> + 'l
    where
        P: AsRef<str> + Clone + 'l,
    {
        let permutations = self
            .generate_source_permutations(langid, res_ids.clone())
            .map(move |sources| sources.into_iter().zip(res_ids.clone()));

        stream::iter(permutations).filter_map(move |sources| async move {
            let mut bundle = FluentBundle::new(&[langid.clone()]);
            for (source, res_id) in sources {
                if let Some(res) = source.fetch_file(&langid, res_id.as_ref()).await {
                    bundle.add_resource(res).unwrap();
                } else {
                    return None;
                }
            }
            Some(bundle)
        })
    }

    pub fn generate_bundles<'l, P>(
        &'l self,
        lang_ids: impl IntoIterator<Item = &'l LanguageIdentifier> + 'l,
        res_ids: impl IntoIterator<Item = P> + Clone + 'l,
    ) -> impl Stream<Item = FluentBundle> + 'l
    where
        P: AsRef<str> + Clone + 'l,
    {
        stream::iter(lang_ids)
            .map(move |langid| self.generate_bundles_for_lang(langid, res_ids.clone()))
            .flatten()
    }
}
