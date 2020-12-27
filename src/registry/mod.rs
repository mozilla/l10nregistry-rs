use crate::bundles::{GenerateBundles, GenerateBundlesSync};
use crate::source::FileSource;
use fluent_bundle::{FluentBundle, FluentResource};
use std::rc::Rc;
use unic_langid::LanguageIdentifier;

pub struct L10nRegistry {
    pub sources: Vec<FileSource>,
}

impl L10nRegistry {
    pub fn new() -> Self {
        Self { sources: vec![] }
    }

    pub fn register_sources(&mut self, mut sources: Vec<FileSource>) {
        self.sources.append(&mut sources);
    }

    // Those two can mess up with `get_bundle_from_order`
    // fn update_sources(&self) {}
    // fn remove_sources(&self) {}
    // fn clear_sources(&self) {}
    // fn has_source(&self) {}

    // fn on_change(&self) {}
    // fn get_available_locales(&self) {}

    pub fn generate_bundles(
        &self,
        locales: Vec<LanguageIdentifier>,
        res_ids: Vec<String>,
        adapt_bundle: Option<fn(&mut FluentBundle<Rc<FluentResource>>)>,
    ) -> GenerateBundles {
        GenerateBundles::new(self, locales, res_ids, adapt_bundle)
    }

    pub fn generate_bundles_sync(
        &self,
        locales: Vec<LanguageIdentifier>,
        res_ids: Vec<String>,
        adapt_bundle: Option<fn(&mut FluentBundle<Rc<FluentResource>>)>,
    ) -> GenerateBundlesSync {
        GenerateBundlesSync::new(self, locales, res_ids, adapt_bundle)
    }
}
