use crate::store::Stores;
use canvas::Resource;
use pigment::Selector;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

use super::QueryResult;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Query {
    pub(crate) count: Option<usize>,

    pub(crate) text: Option<String>,
}

impl Query {
    pub fn execute(&self, stores: &Stores) -> Vec<QueryResult> {
        todo!()
    }
}

impl<R: Resource> Selector<R> for Query {
    fn matches(&self, _resource: &R) -> bool {
        // FIXME: implement query matching
        true
    }
}

pub trait Score<R: Resource> {
    fn score(&self, resource: &R) -> isize;
}

// FIXME: implement query ordering
impl<R: Resource> Score<R> for Query {
    fn score(&self, _resource: &R) -> isize {
        0
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TS_QUERY: &str = r#"
export type Query = {
    count?: number;

    text?: string;
};
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Query")]
    pub type JsQuery;
}
