use fluent_bundle::FluentBundle as FluentBundleBase;
pub use fluent_bundle::FluentResource;
use std::rc::Rc;

pub type FluentBundle = FluentBundleBase<Rc<FluentResource>>;
