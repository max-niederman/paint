pub mod query;

use crate::store::{self, Stores};
use chrono::{DateTime, Utc};
use futures::future;
use js_sys::Promise;
use pigment::View;
use query::JsQuery;
pub use query::Query;
use std::{fmt::Display, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen]
pub struct SearchManager {
    pub(crate) stores: Rc<Stores>,
}

#[wasm_bindgen]
impl SearchManager {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<SearchManager, JsValue> {
        tracing::info!("constructing search manager");
        Ok(SearchManager {
            stores: Rc::new(Stores::new().await?),
        })
    }

    // TODO: the typescript definition should know this returns a Result<QueryResult, JsValue>
    /// Run the given query in the given view.
    pub fn query(&self, view: &JsView, query: &JsQuery) -> Result<JsValue, JsValue> {
        // we use serde-wasm-bindgen because the `View` type contains a bigint
        let view: View = serde_wasm_bindgen::from_value(view.into()).map_err(into_exception)?;
        let query: Query = query.into_serde().map_err(into_exception)?;

        JsValue::from_serde(&query.execute(&self.stores).map_err(into_exception)?)
            .map_err(into_exception)
    }

    /// Update the views in the store.
    pub fn update(&self, view: &JsView, since: &str) -> Result<Promise, JsValue> {
        let stores = self.stores.clone();
        // we use serde-wasm-bindgen because the `View` type contains a bigint
        let view = serde_wasm_bindgen::from_value(view.into()).map_err(into_exception)?;
        let since = since.parse::<DateTime<Utc>>().map_err(into_exception)?;

        Ok(future_to_promise(async move {
            store::oil::update_stores(&stores, since, &view)
                .await
                .map_err(into_exception)?;
            tracing::info!(message = "finished store update", %view);
            Ok(JsValue::UNDEFINED)
        }))
    }

    /// Save the current store to IndexedDB.
    pub fn save(&self) -> Promise {
        let stores = self.stores.clone();
        future_to_promise(async move {
            future::try_join_all([
                stores.courses.write(),
                stores.assignments.write(),
                stores.submissions.write(),
            ])
            .await?;
            Ok::<_, JsValue>(JsValue::UNDEFINED)
        })
    }
}

fn into_exception(err: impl Display) -> JsValue {
    JsValue::from_str(&format!("{}", err))
}

#[wasm_bindgen(typescript_custom_section)]
const TS_VIEW: &str = r#"
export type View = {
    truth: { base_url: string; };
    viewer: { User: bigint; };
};
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "View")]
    pub type JsView;
}
