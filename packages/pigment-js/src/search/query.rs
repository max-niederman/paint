use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Query {
    pub(crate) text: Option<String>,
}
