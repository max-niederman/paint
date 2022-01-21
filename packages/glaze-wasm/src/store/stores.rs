use super::GlazeStore;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub struct Stores {
    pub(crate) courses: GlazeStore,
    pub(crate) assignments: GlazeStore,
    pub(crate) submissions: GlazeStore,
}

impl Stores {
    pub async fn new() -> Result<Stores, JsValue> {
        Ok(Self {
            courses: GlazeStore::load("courses").await?,
            assignments: GlazeStore::load("assignments").await?,
            submissions: GlazeStore::load("submissions").await?,
        })
    }
}
