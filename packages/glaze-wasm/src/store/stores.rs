use super::GlazeStore;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
#[wasm_bindgen]
pub struct Stores {
    pub(crate) courses: GlazeStore,
    pub(crate) assignments: GlazeStore,
    pub(crate) submissions: GlazeStore,
}

#[wasm_bindgen]
impl Stores {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<Stores, JsValue> {
        Ok(Self {
            courses: GlazeStore::load("courses").await?,
            assignments: GlazeStore::load("assignments").await?,
            submissions: GlazeStore::load("submissions").await?,
        })
    }
}
