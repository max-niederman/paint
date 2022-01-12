use super::Query;
use crate::store::GlazeStore;
use wasm_bindgen::prelude::*;

use web_sys::DomException;

#[wasm_bindgen]
#[derive(Debug)]
pub struct Manager {
    stores: Stores,
    query: Query,
    subscribers: Vec<js_sys::Function>,
}

#[wasm_bindgen]
impl Manager {
    /// Construct a new query manager.
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<Manager, DomException> {
        Ok(Self {
            stores: Stores::new().await?,
            query: Query::default(),
            subscribers: Vec::new(),
        })
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

#[derive(Debug)]
struct Stores {
    courses: GlazeStore,
    assignments: GlazeStore,
    submissions: GlazeStore,
}

impl Stores {
    async fn new() -> Result<Self, DomException> {
        Ok(Self {
            courses: GlazeStore::load("courses").await?,
            assignments: GlazeStore::load("assignments").await?,
            submissions: GlazeStore::load("submissions").await?,
        })
    }
}
