pub mod query;

use crate::store::{self, Stores};
use canvas::resource::*;
use chrono::{DateTime, Utc};
use js_sys::Promise;
use pigment::View;
use query::JsQuery;
pub use query::Query;
use serde::Serialize;
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

        JsValue::from_serde(&query.execute(&self.stores)).map_err(into_exception)
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
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "resource")]
pub enum QueryResult {
    Course(Course),
    Assignment(Assignment),
    Submission(Submission),
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


// TODO: add stricter types for resources
#[wasm_bindgen(typescript_custom_section)]
const TS_QUERY_RESULTS: &str = r#"
export type QueryResult = 
    | {
        type: "course";
        resource: any;
      }
    | {
        type: "assignment";
        resource: any;
      }
    | {
        type: "submission";
        resource: any;
      };

export type QueryResults = QueryResult[];
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "View")]
    pub type JsView;
}
