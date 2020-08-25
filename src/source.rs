use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;

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

async fn read_resource<P: AsRef<Path>>(path: P) -> ResourceOption {
    gecko::fetch(path.as_ref())
        .await
        .ok()
        .map(|source| Arc::new(FluentResource { source }))
}

fn set_resolved(cache: Rc<RefCell<HashMap<PathBuf, ResourceStatus>>>, full_path: PathBuf, value: &ResourceOption) {
    let mut cache = cache.try_borrow_mut().unwrap();
    if let Some(res) = value {
        cache.insert(full_path, ResourceStatus::Sync(Some(res.clone())));
    } else {
        cache.insert(full_path, ResourceStatus::Sync(None));
    }
}

pub struct FileSource {
    pub name: String,
    pub langids: Vec<LanguageIdentifier>,
    pub pre_path: PathBuf,
    pub cache: Rc<RefCell<HashMap<PathBuf, ResourceStatus>>>,
}

impl FileSource {
    pub fn new(name: String, langids: Vec<LanguageIdentifier>, pre_path: PathBuf) -> Self {
        FileSource {
            name,
            langids,
            pre_path,
            cache: Rc::new(RefCell::new(HashMap::default())),
        }
    }

    pub fn fetch_file_sync(&self, langid: &LanguageIdentifier, path: &Path) -> ResourceOption {
        let full_path = self.pre_path.resolve_path(langid).join(path);

        let mut cache = self.cache.try_borrow_mut().unwrap();
        let res = cache.entry(full_path.clone()).or_insert_with(|| {
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
                cache.remove(&full_path);
                drop(cache);
                self.fetch_file_sync(langid, path)
            }
        }
    }

    pub async fn fetch_file(&self, langid: &LanguageIdentifier, path: &Path) -> ResourceOption {
        let full_path = self.pre_path.resolve_path(langid).join(path);


        let cache_cell = &self.cache;
        let mut cache = cache_cell.try_borrow_mut().unwrap();
        let cache_cell2 = self.cache.clone();
        let full_path2 = full_path.clone();

        let res = cache.entry(full_path.clone()).or_insert_with(|| {
            ResourceStatus::Async(
                read_resource(full_path)
                .inspect(move |x| {
                    set_resolved(cache_cell2, full_path2, x)}
                )
                .boxed_local()
                .shared()
            )
        }).clone();
        drop(cache);

        match res {
            ResourceStatus::Sync(res) => res.as_ref().cloned(),
            ResourceStatus::Async(res) => {
                res.await
            },
        }
    }

    pub fn has_file(&self, langid: &LanguageIdentifier, path: &Path) -> Option<bool> {
        if !self.langids.contains(langid) {
            return Some(false);
        } else {
            let full_path = self.pre_path.resolve_path(langid).join(path);
            let cache = self.cache.try_borrow().unwrap();
            if let Some(res) = cache.get(&full_path) {
                match res {
                    ResourceStatus::Sync(res) => Some(res.is_some()),
                    ResourceStatus::Async(_) => {
                        // res.clone().inspect(|x| println!("about to resolve: {:#?}", x));
                        // println!("{:#?}", res);
                        // res.peek().map(|ro| ro.is_some())
                        None
                    },
                }
            } else {
                return None;
            }
        }
    }
}
