use super::{GlazeStore, StoreName};

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
            courses: GlazeStore::load(StoreName::Course).await?,
            assignments: GlazeStore::load(StoreName::Assignment).await?,
            submissions: GlazeStore::load(StoreName::Submission).await?,
        })
    }
}
