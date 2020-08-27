use std::rc::Rc;

#[derive(Debug)]
pub struct FluentResource {
    pub source: String,
}

#[derive(Debug)]
pub struct FluentBundle {
    pub resources: Vec<Rc<FluentResource>>,
}
