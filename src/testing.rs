use crate::{FileFetcher, FileSource};
use async_trait::async_trait;
use unic_langid::LanguageIdentifier;
use fluent_testing::MockFileSystem;
use std::rc::Rc;
use crate::registry::L10nRegistry;

#[derive(Default)]
struct InnerFileFetcher {
    fs: MockFileSystem,
}

#[derive(Clone)]
pub struct TestFileFetcher {
    inner: Rc<InnerFileFetcher>,
}

impl TestFileFetcher {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(InnerFileFetcher::default())
        }
    }

    pub fn get_test_file_source(
        &self,
        name: &str,
        locales: Vec<LanguageIdentifier>,
        path: &str,
    ) -> FileSource {
        FileSource::new(name.to_string(), locales, path.to_string(), self.clone())
    }

    pub fn get_registry(&self, scenario: &fluent_testing::scenarios::structs::Scenario) -> L10nRegistry {
        let locales: Vec<LanguageIdentifier> = scenario
            .locales
            .iter()
            .map(|l| l.parse().unwrap())
            .collect();

        let mut reg = L10nRegistry::default();
        let sources = scenario
            .file_sources
            .iter()
            .map(|source| {
                self.get_test_file_source(
                    &source.name,
                    source.locales.iter().map(|s| s.parse().unwrap()).collect(),
                    &source.path_scheme,
                    )
            })
        .collect();
        reg.register_sources(sources).unwrap();
        reg.set_lang_ids(locales);
        reg
    }
}

#[async_trait(?Send)]
impl FileFetcher for TestFileFetcher {
    fn fetch_sync(&self, path: &str) -> std::io::Result<String> {
        self.inner.fs.get_test_file_sync(path)
    }

    async fn fetch(&self, path: &str) -> std::io::Result<String> {
        self.inner.fs.get_test_file_async(path).await
    }
}
