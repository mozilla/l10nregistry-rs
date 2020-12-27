use crate::{source::FileFetcher, FileSource};
use async_trait::async_trait;
use unic_langid::LanguageIdentifier;

pub struct TestFileFetcher;

#[async_trait]
impl FileFetcher for TestFileFetcher {
    fn fetch_sync(&self, path: &str) -> std::io::Result<String> {
        fluent_testing::get_test_file_sync(path)
    }

    async fn fetch(&self, path: &str) -> std::io::Result<String> {
        println!("{:#?}", path);
        fluent_testing::get_test_file_async(path).await
    }
}

pub fn get_test_file_source(
    name: &str,
    locales: Vec<LanguageIdentifier>,
    path: &str,
) -> FileSource {
    FileSource::new(name.to_string(), locales, path.to_string(), TestFileFetcher)
}
