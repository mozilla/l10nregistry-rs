use async_trait::async_trait;
use std::io;

#[async_trait]
pub trait FileFetcher {
    fn fetch_sync(&self, path: &str) -> io::Result<String>;
    async fn fetch(&self, path: &str) -> io::Result<String>;
}
