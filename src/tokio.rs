use async_trait::async_trait;
use tokio::io::{self, AsyncReadExt};
use unic_langid::LanguageIdentifier;

use crate::{source::FileSource, FileFetcher};

pub struct TokioFileFetcher;

#[async_trait]
impl FileFetcher for TokioFileFetcher {
    fn fetch_sync(&self, path: &str) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    async fn fetch(&self, path: &str) -> io::Result<String> {
        let mut f = ::tokio::fs::File::open(path).await?;
        let mut s = String::new();
        f.read_to_string(&mut s).await?;
        println!("read = {}", s);
        Ok(s)
    }
}

pub fn file_source(name: String, langids: Vec<LanguageIdentifier>, pre_path: String) -> FileSource {
    FileSource::new(name, langids, pre_path, TokioFileFetcher)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fetch_string_present() {
        let fetcher = TokioFileFetcher;
        let s = fetcher
            .fetch("./tests/resources/toolkit/en-US/toolkit/menu.ftl".into())
            .await;
        assert!(s.is_ok());
        let s = s.unwrap();
        assert_eq!(s.len(), 82);
    }

    #[tokio::test]
    async fn fetch_string_missing() {
        let fetcher = TokioFileFetcher;
        let s = fetcher
            .fetch("./tests/resources/toolkit/en-US/toolkit/brand.ftl".into())
            .await;
        assert!(s.is_err());
    }
}
