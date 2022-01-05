use wasm_bindgen::prelude::*;
use super::Query;

#[wasm_bindgen]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Manager {
    query: Query,
    subscribers: Vec<js_sys::Function>,
}

#[wasm_bindgen]
impl Manager {
    /// Construct a new query manager.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe a callback to the query result.
    pub fn subscribe(&mut self, f: js_sys::Function) {
        self.subscribers.push(f);
    }

    #[wasm_bindgen(js_name = "setQueryText")]
    pub fn set_query_text(&mut self, text: Option<String>) {
        self.query.text = text;
    }
}