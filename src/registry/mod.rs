mod asynchronous;
mod synchronous;

use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use crate::errors::L10nRegistrySetupError;
use crate::source::FileSource;

use crate::environment::{ErrorReporter, LocalesProvider};
use crate::fluent::FluentBundle;
use fluent_bundle::FluentResource;
use fluent_fallback::generator::BundleGenerator;
use unic_langid::LanguageIdentifier;

pub use asynchronous::GenerateBundles;
pub use synchronous::GenerateBundlesSync;

pub type FluentResourceSet = Vec<Rc<FluentResource>>;

#[derive(Default)]
struct Shared<P> {
    sources: RefCell<Vec<FileSource>>,
    provider: P,
    adapt_bundle: Option<fn(&mut FluentBundle)>,
}

pub struct L10nRegistryLocked<'a> {
    lock: Ref<'a, Vec<FileSource>>,
    adapt_bundle: Option<fn(&mut FluentBundle)>,
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
        let source_idx = self.len() - 1 - index;
        self.lock.get(source_idx).expect("Index out-of-range")
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

#[derive(Clone)]
pub struct L10nRegistry<P> {
    shared: Rc<Shared<P>>,
}

impl<P> L10nRegistry<P> {
    pub fn with_provider(provider: P) -> Self {
        Self {
            shared: Rc::new(Shared {
                sources: Default::default(),
                provider,
                adapt_bundle: None,
            }),
        }
    }

    pub fn set_adapt_bundle(
        &mut self,
        adapt_bundle: fn(&mut FluentBundle),
    ) -> Result<(), L10nRegistrySetupError> {
        let shared = Rc::get_mut(&mut self.shared).ok_or(L10nRegistrySetupError::RegistryLocked)?;
        shared.adapt_bundle = Some(adapt_bundle);
        Ok(())
    }

    pub fn lock(&self) -> L10nRegistryLocked<'_> {
        L10nRegistryLocked {
            lock: self.shared.sources.borrow(),
            adapt_bundle: self.shared.adapt_bundle,
        }
    }

    pub fn register_sources(
        &mut self,
        new_sources: Vec<FileSource>,
    ) -> Result<(), L10nRegistrySetupError> {
        let shared = Rc::get_mut(&mut self.shared).ok_or(L10nRegistrySetupError::RegistryLocked)?;
        let sources = shared.sources.get_mut();

        for new_source in new_sources {
            if sources.iter().any(|source| source == &new_source) {
                return Err(L10nRegistrySetupError::DuplicatedSource {
                    name: new_source.name,
                });
            }
            sources.push(new_source);
        }
        Ok(())
    }

    pub fn update_sources(
        &mut self,
        upd_sources: Vec<FileSource>,
    ) -> Result<(), L10nRegistrySetupError> {
        let shared = Rc::get_mut(&mut self.shared).ok_or(L10nRegistrySetupError::RegistryLocked)?;
        let sources = shared.sources.get_mut();

        for upd_source in upd_sources {
            if let Some(idx) = sources.iter().position(|source| *source == upd_source) {
                *sources.get_mut(idx).unwrap() = upd_source;
            } else {
                return Err(L10nRegistrySetupError::MissingSource {
                    name: upd_source.name,
                });
            }
        }
        Ok(())
    }

    pub fn remove_sources(&mut self, del_sources: Vec<&str>) -> Result<(), L10nRegistrySetupError> {
        let shared = Rc::get_mut(&mut self.shared).ok_or(L10nRegistrySetupError::RegistryLocked)?;
        let sources = shared.sources.get_mut();

        sources.retain(|source| !del_sources.contains(&source.name.as_str()));
        Ok(())
    }
}

impl<P> BundleGenerator for L10nRegistry<P>
where
    P: LocalesProvider + ErrorReporter + Clone,
{
    type Resource = Rc<FluentResource>;
    type Iter = GenerateBundlesSync<P>;
    type Stream = GenerateBundles<P>;

    fn bundles_stream(&self, resource_ids: Vec<String>) -> Self::Stream {
        self.generate_bundles(self.shared.provider.locales().to_vec(), resource_ids)
    }

    fn bundles_iter(&self, resource_ids: Vec<String>) -> Self::Iter {
        self.generate_bundles_sync(self.shared.provider.locales().to_vec(), resource_ids)
    }
}
