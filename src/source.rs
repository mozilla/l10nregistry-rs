use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use futures::future::{FutureExt, Shared};
use unic_langid::LanguageIdentifier;

use crate::fluent::FluentResource;
use crate::gecko;
use crate::uri::ResourceURI;

pub type ResourceOption = Option<Arc<FluentResource>>;

#[derive(Debug, Clone)]
pub enum ResourceStatus {
    Sync(ResourceOption),
    Async(Shared<Pin<Box<dyn Future<Output = ResourceOption>>>>),
}

pub struct FileSource {
    pub name: String,
    pub langids: Vec<LanguageIdentifier>,
    pub pre_path: PathBuf,
    pub cache: HashMap<PathBuf, ResourceStatus>,
}

async fn read_resource<P: AsRef<Path>>(path: P) -> ResourceOption {
    gecko::fetch(path.as_ref())
        .await
        .ok()
        .map(|source| Arc::new(FluentResource { source }))
}

impl FileSource {
    pub fn new(name: String, langids: Vec<LanguageIdentifier>, pre_path: PathBuf) -> Self {
        FileSource {
            name,
            langids,
            pre_path,
            cache: HashMap::default(),
        }
    }

    pub fn fetch_file_sync(&mut self, langid: &LanguageIdentifier, path: &Path) -> ResourceOption {
        let full_path = self.pre_path.resolve_path(langid).join(path);

        let res = self.cache.entry(full_path.clone()).or_insert_with(|| {
            ResourceStatus::Sync(
                std::fs::read_to_string(&full_path)
                    .ok()
                    .map(|source| Arc::new(FluentResource { source })),
            )
        });

        match res {
            ResourceStatus::Sync(res_opt) => res_opt.as_ref().cloned(),
            ResourceStatus::Async(..) => {
                println!("[l10nregistry] Attempting to synchronously load file {} while it's being loaded asynchronously.", full_path.to_string_lossy());
                self.cache.remove(&full_path);
                self.fetch_file_sync(langid, path)
            }
        }
    }

    pub async fn fetch_file(&mut self, langid: &LanguageIdentifier, path: &Path) -> ResourceOption {
        let full_path = self.pre_path.resolve_path(langid).join(path);

        let res = self.cache.entry(full_path.clone()).or_insert_with(|| {
            ResourceStatus::Async(read_resource(full_path).boxed_local().shared())
        });

        match res {
            ResourceStatus::Sync(res) => res.as_ref().cloned(),
            ResourceStatus::Async(res) => res.await,
        }
    }

}
