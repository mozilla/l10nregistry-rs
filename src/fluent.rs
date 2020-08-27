use fluent::FluentBundle as FluentBundleBase;
pub use fluent::FluentResource;
use std::rc::Rc;

pub type FluentBundle = FluentBundleBase<Rc<FluentResource>>;
