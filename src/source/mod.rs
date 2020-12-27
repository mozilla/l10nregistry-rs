mod fetcher;

pub use fetcher::FileFetcher;
use fluent_bundle::FluentResource;
use futures::{future::Shared, Future, FutureExt};
use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;
use unic_langid::LanguageIdentifier;

pub type RcResource = Rc<FluentResource>;
pub type RcResourceOption = Option<RcResource>;
pub type ResourceFuture = Shared<Pin<Box<dyn Future<Output = RcResourceOption>>>>;

#[derive(Clone)]
pub enum ResourceStatus {
    Missing,
    Loading(ResourceFuture),
    Loaded(RcResource),
}

impl From<RcResourceOption> for ResourceStatus {
    fn from(input: RcResourceOption) -> Self {
        if let Some(res) = input {
            Self::Loaded(res)
        } else {
            Self::Missing
        }
    }
}

impl Future for ResourceStatus {
    type Output = bool;

    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        use ResourceStatus::*;

        match &*self {
            Missing => false.into(),
            Loaded(_) => true.into(),
            Loading(_) => std::task::Poll::Pending,
        }
    }
}

pub struct FileSource {
    _name: String,
    _locales: Vec<LanguageIdentifier>,
    pre_path: String,
    inner: Rc<InnerFileSource>,
}

impl FileSource {
    pub fn new(
        name: String,
        locales: Vec<LanguageIdentifier>,
        pre_path: String,
        fetcher: impl FileFetcher + 'static,
    ) -> Self {
        FileSource {
            _name: name,
            _locales: locales,
            pre_path,
            inner: Rc::new(InnerFileSource {
                entries: RefCell::new(HashMap::default()),
                fetcher: Box::new(fetcher),
            }),
        }
    }

    fn get_path(&self, langid: &LanguageIdentifier, path: &str) -> String {
        format!(
            "{}{}",
            self.pre_path.replace("{locale}", &langid.to_string()),
            path
        )
    }

    fn fetch_sync(&self, full_path: &str) -> RcResourceOption {
        self.inner
            .fetcher
            .fetch_sync(full_path)
            .ok()
            .and_then(|source| FluentResource::try_new(source).ok())
            .map(Rc::new)
    }

    pub fn fetch_file_sync(&self, langid: &LanguageIdentifier, path: &str) -> RcResourceOption {
        let full_path = self.get_path(langid, &path);

        let res = self
            .inner
            .lookup_resource(&full_path, || self.fetch_sync(&full_path).into());

        match res {
            ResourceStatus::Missing => None,
            ResourceStatus::Loaded(res) => Some(res),
            ResourceStatus::Loading(..) => self.fetch_sync(&full_path),
        }
    }

    pub fn fetch_file(&self, langid: &LanguageIdentifier, path: &str) -> ResourceStatus {
        use ResourceStatus::*;

        let full_path = self.get_path(langid, path);

        self.inner.lookup_resource(&full_path, || {
            let shared = self.inner.clone();
            Loading(
                read_resource(full_path.clone(), shared)
                    .boxed_local()
                    .shared(),
            )
        })
    }
}

async fn read_resource(path: String, inner: Rc<InnerFileSource>) -> RcResourceOption {
    let resource =
        inner.fetcher.fetch(&path).await.ok().map(|source| {
            Rc::new(FluentResource::try_new(source).expect("Failed to parse source"))
        });
    // insert the resource into the cache
    inner.update_resource(&path, resource)
}

struct InnerFileSource {
    fetcher: Box<dyn FileFetcher>,
    entries: RefCell<HashMap<String, ResourceStatus>>,
}

impl InnerFileSource {
    fn lookup_resource<F>(&self, path: &str, f: F) -> ResourceStatus
    where
        F: FnOnce() -> ResourceStatus,
    {
        let mut lock = self.entries.borrow_mut();
        lock.entry(path.to_string()).or_insert_with(|| f()).clone()
    }

    fn update_resource(&self, path: &str, resource: RcResourceOption) -> RcResourceOption {
        let mut lock = self.entries.borrow_mut();
        let entry = lock.get_mut(path);
        match entry {
            Some(entry) => *entry = resource.clone().into(),
            _ => panic!("Expected "),
        }
        resource
    }
}
