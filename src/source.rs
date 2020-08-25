use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::rc::Rc;

use futures::future::{FutureExt, Shared};
use unic_langid::LanguageIdentifier;

use crate::fluent::FluentResource;
use crate::gecko;
use crate::uri::ResourceURI;

pub type RcResource = Rc<FluentResource>;
pub type ResourceOption = Option<RcResource>;

#[derive(Debug, Clone)]
pub enum ResourceStatus {
    Value(RcResource),
    Async(Shared<Pin<Box<dyn Future<Output = ResourceOption>>>>),
    None,
}

async fn read_resource<P: AsRef<Path>>(path: P) -> ResourceOption {
    gecko::fetch(path.as_ref())
        .await
        .ok()
        .map(|source| Rc::new(FluentResource { source }))
}

fn set_resolved(
    cache: Rc<RefCell<HashMap<PathBuf, ResourceStatus>>>,
    full_path: PathBuf,
    value: &ResourceOption,
) {
    println!("Set Resolved Called");
    let mut cache = cache.try_borrow_mut().unwrap();
    if let Some(res) = value {
        cache.insert(full_path, ResourceStatus::Value(res.clone()));
    } else {
        cache.insert(full_path, ResourceStatus::None);
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
            if let Ok(source) = gecko::fetch_sync(&full_path) {
                ResourceStatus::Value(Rc::new(FluentResource { source }))
            } else {
                ResourceStatus::None
            }
        });

        match res {
            ResourceStatus::Value(res) => Some(res.clone()),
            ResourceStatus::Async(..) => {
                println!("[l10nregistry] Attempting to synchronously load file {} while it's being loaded asynchronously.", full_path.to_string_lossy());
                cache.remove(&full_path);
                drop(cache);
                self.fetch_file_sync(langid, path)
            }
            ResourceStatus::None => None,
        }
    }

    pub async fn fetch_file(&self, langid: &LanguageIdentifier, path: &Path) -> ResourceOption {
        let full_path = self.pre_path.resolve_path(langid).join(path);

        let cache_cell = &self.cache;
        let mut cache = cache_cell.try_borrow_mut().unwrap();
        let cloned_full_path = full_path.clone();

        let res = cache
            .entry(full_path.clone())
            .or_insert_with(|| {
                let cache = self.cache.clone();
                ResourceStatus::Async(
                    read_resource(full_path)
                        // We inspect here to extract the result of the async call
                        // and put it into the cache.
                        // This allows for `has_file` to synchronusly verify that a path
                        // has no file once the future is resolved.
                        //
                        // This requires extranous locks on the cache, and I'd like to
                        // try to use `MaybeDone` instead to check on a `Async` variant of the
                        // `ResourceStatus` and see if it has been resolved.
                        // My initial attempt to use it didn't work, but that may be just
                        // my lack of experience with the API.
                        .inspect(|res| set_resolved(cache, cloned_full_path, res))
                        .boxed_local()
                        .shared(),
                )
            })
            .clone();
        drop(cache);

        match res {
            ResourceStatus::Value(res) => Some(res),
            ResourceStatus::Async(res) => res.await,
            ResourceStatus::None => None,
        }
    }

    pub fn has_file(&self, langid: &LanguageIdentifier, path: &Path) -> Option<bool> {
        if !self.langids.contains(langid) {
            Some(false)
        } else {
            let full_path = self.pre_path.resolve_path(langid).join(path);
            let cache = self.cache.try_borrow().unwrap();
            if let Some(res) = cache.get(&full_path) {
                match res {
                    ResourceStatus::Value(_) => Some(true),
                    ResourceStatus::Async(_) => None,
                    ResourceStatus::None => Some(false),
                }
            } else {
                None
            }
        }
    }
}
