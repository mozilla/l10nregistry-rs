mod asynchronous;
mod synchronous;

use std::{
    cell::{Ref, RefCell},
    iter::Rev,
    ops::Range,
    rc::Rc,
};

use crate::source::FileSource;

use chunky_vec::ChunkyVec;
use fluent_bundle::FluentResource;
use fluent_fallback::{BundleGenerator, BundleGeneratorSync};
use itertools::Itertools;
use unic_langid::LanguageIdentifier;

pub use asynchronous::GenerateBundles;
pub use synchronous::GenerateBundlesSync;

pub type FluentResourceSet = Vec<Rc<FluentResource>>;

/// Generate a permutation of all registered source file indices for `length`
/// in reverse order. ie. The last source added to the registry with `add_source`
/// is returned first.
pub(crate) fn permute_iter(
    source_count: usize,
    length: usize,
) -> itertools::MultiProduct<Rev<Range<usize>>> {
    (0..length)
        .map(|_| (0..source_count).rev())
        .multi_cartesian_product()
}

#[derive(Default)]
struct Shared {
    lang_ids: Vec<LanguageIdentifier>,
    sources: RefCell<ChunkyVec<FileSource>>,
}

pub struct L10nRegistryLocked<'a> {
    lock: Ref<'a, ChunkyVec<FileSource>>,
}

impl<'a> L10nRegistryLocked<'a> {
    pub fn iter(&self) -> impl Iterator<Item = &FileSource> {
        self.lock.iter()
    }

    pub fn len(&self) -> usize {
        self.lock.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn source_idx(&self, index: usize) -> &FileSource {
        self.lock.get(index).expect("Index out-of-range")
    }

    pub fn get_source(&self, name: &str) -> Option<&FileSource> {
        self.lock.iter().find(|&source| source.name == name)
    }

    pub fn generate_sources_for_file<'l>(
        &'l self,
        langid: &'l LanguageIdentifier,
        res_id: &'l str,
    ) -> impl Iterator<Item = &FileSource> {
        self.iter()
            .filter(move |source| source.has_file(langid, res_id) != Some(false))
    }
}

#[derive(Clone, Default)]
pub struct L10nRegistry {
    shared: Rc<Shared>,
}

impl L10nRegistry {
    pub fn lock(&self) -> L10nRegistryLocked<'_> {
        L10nRegistryLocked {
            lock: self.shared.sources.borrow(),
        }
    }

    pub fn register_sources(&mut self, new_sources: Vec<FileSource>) -> Result<(), ()> {
        let shared = Rc::get_mut(&mut self.shared).unwrap();
        let sources = shared.sources.get_mut();

        for new_source in new_sources {
            if sources.iter().any(|source| source == &new_source) {
                return Err(());
            }
            sources.push(new_source);
        }
        Ok(())
    }

    pub fn set_lang_ids(&mut self, lang_ids: impl IntoIterator<Item = LanguageIdentifier>) {
        let shared = Rc::get_mut(&mut self.shared).unwrap();
        shared.lang_ids = lang_ids.into_iter().collect::<Vec<_>>();
    }
}

impl BundleGenerator for L10nRegistry {
    type Resource = Rc<FluentResource>;
    type Stream = GenerateBundles;

    fn bundles(&self, resource_ids: Vec<String>) -> Self::Stream {
        self.generate_bundles(self.shared.lang_ids.clone(), resource_ids)
    }
}

impl BundleGeneratorSync for L10nRegistry {
    type Resource = Rc<FluentResource>;
    type Iter = GenerateBundlesSync;

    fn bundles_sync(&self, resource_ids: Vec<String>) -> Self::Iter {
        self.generate_bundles_sync(self.shared.lang_ids.clone(), resource_ids)
    }
}

#[cfg(test)]
#[cfg(feature = "tokio")]
mod tests {
    use super::*;

    fn test_setup_registry(reg: &mut L10nRegistry) {
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let fs1 = crate::tokio::file_source(
            "toolkit".to_string(),
            vec![en_us.clone()],
            "./data/toolkit/{locale}".into(),
        );
        let fs2 = crate::tokio::file_source(
            "browser".to_string(),
            vec![en_us.clone()],
            "./data/browser/{locale}".into(),
        );

        reg.register_sources(vec![fs1, fs2]).unwrap();
    }

    #[test]
    fn permutations() {
        let mut reg = L10nRegistry::default();
        test_setup_registry(&mut reg);

        let mut iter = permute_iter(reg.lock().len(), 2);
        assert_eq!(iter.next(), Some(vec![1, 1]));
        assert_eq!(iter.next(), Some(vec![1, 0]));
        assert_eq!(iter.next(), Some(vec![0, 1]));
        assert_eq!(iter.next(), Some(vec![0, 0]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn generate_resource_set_sync() {
        let mut reg = L10nRegistry::default();
        test_setup_registry(&mut reg);

        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let resource_ids = ["brand.ftl", "menu.ftl"];
        for (i, order) in permute_iter(reg.lock().len(), resource_ids.len()).enumerate() {
            let set = reg
                .lock()
                .generate_resource_set_sync(&en_us, &order, &resource_ids);
            if i == 1 {
                assert!(set.is_some());
                let set = set.unwrap();
                assert_eq!(set.len(), 2);
            } else {
                assert!(set.is_none());
            }
        }
    }

    #[tokio::test]
    async fn generate_resource_set_async() {
        let mut reg = L10nRegistry::default();
        test_setup_registry(&mut reg);

        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let resource_ids = ["brand.ftl", "menu.ftl"];
        for (i, order) in permute_iter(reg.lock().len(), resource_ids.len()).enumerate() {
            let set = reg
                .lock()
                .generate_resource_set(&en_us, &order, &resource_ids)
                .await;
            if i == 1 {
                assert!(set.is_some());
                let set = set.unwrap();
                assert_eq!(set.len(), 2);
            } else {
                assert!(set.is_none());
            }
        }
    }
}
