use fluent_bundle::FluentBundle as FluentBundleBase;
pub use fluent_bundle::{FluentResource, FluentError};
use std::rc::Rc;

pub type FluentBundle = FluentBundleBase<Rc<FluentResource>>;
