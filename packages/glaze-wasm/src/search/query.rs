use canvas::Resource;
use pigment::Selector;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Query {
    pub(crate) text: Option<String>,
}

impl<R: Resource> Selector<R> for Query {
    fn matches(&self, resource: &R) -> bool {
        // FIXME: implement query matching
        true
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TS_QUERY: &str = r#"
export type Query = {
    text?: string;
};
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Query")]
    pub type JsQuery;
}
