use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::hash::{Hash, Hasher};
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

impl From<ResourceOption> for ResourceStatus {
    fn from(input: ResourceOption) -> Self {
        if let Some(res) = input {
            Self::Value(res)
        } else {
            Self::None
        }
    }
}

async fn read_resource<P: AsRef<Path>>(path: P) -> ResourceOption {
    gecko::fetch(path.as_ref())
        .await
        .ok()
        .map(|source| Rc::new(FluentResource::try_new(source).unwrap()))
}

fn set_resolved(
    cache: Rc<RefCell<HashMap<PathBuf, ResourceStatus>>>,
    full_path: PathBuf,
    value: ResourceOption,
) {
    cache
        .try_borrow_mut()
        .unwrap()
        .insert(full_path, value.into());
}

pub struct FileSource {
    pub name: String,
    pub langids: Vec<LanguageIdentifier>,
    pub pre_path: PathBuf,
    pub cache: Rc<RefCell<HashMap<PathBuf, ResourceStatus>>>,
    pub fetch_sync: fn(&Path) -> Result<Option<String>, std::io::Error>,
}

impl fmt::Display for FileSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq<FileSource> for FileSource {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for FileSource {}

impl Hash for FileSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl FileSource {
    pub fn new(
        name: String,
        langids: Vec<LanguageIdentifier>,
        pre_path: PathBuf,
        fetch_sync: fn(&Path) -> Result<Option<String>, std::io::Error>,
    ) -> Self {
        FileSource {
            name,
            langids,
            pre_path,
            cache: Rc::new(RefCell::new(HashMap::default())),
            fetch_sync,
        }
    }

    fn get_path(&self, langid: &LanguageIdentifier, path: &Path) -> PathBuf {
        self.pre_path.resolve_path(langid).join(path)
    }

    pub fn fetch_file_sync(&self, langid: &LanguageIdentifier, path: &Path) -> ResourceOption {
        let full_path = self.get_path(langid, path);

        let mut cache = self.cache.try_borrow_mut().unwrap();
        let res = cache.entry(full_path.clone()).or_insert_with(|| {
            (self.fetch_sync)(&full_path)
                .expect("I/O Error")
                .map(|source| Rc::new(FluentResource::try_new(source).unwrap()))
                .into()
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
        let full_path = self.get_path(langid, path);

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
                        .inspect(|res| set_resolved(cache, cloned_full_path, res.clone()))
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

    pub fn has_file<L: Borrow<LanguageIdentifier>>(&self, langid: L, path: &Path) -> Option<bool> {
        let langid = langid.borrow();
        if !self.langids.contains(langid) {
            Some(false)
        } else {
            let full_path = self.get_path(langid, path);
            let cache = self.cache.try_borrow().unwrap();
            match cache.get(&full_path) {
                Some(ResourceStatus::Value(_)) => Some(true),
                Some(ResourceStatus::None) => Some(false),
                Some(ResourceStatus::Async(_)) | None => None,
            }
        }
    }
}

impl std::fmt::Debug for FileSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileSource")
            .field("name", &self.name)
            .field("langids", &self.langids)
            .field("pre_path", &self.pre_path)
            .field("cache", &self.cache)
            .finish()
    }
}
