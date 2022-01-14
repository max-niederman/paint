use canvas::Resource;
use pigment::Selector;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Query {
    pub(crate) text: Option<String>,
}

impl<R: Resource> Selector<R> for Query {
    fn matches(&self, resource: &R) -> bool {
        todo!("implement query matching")
    }
}