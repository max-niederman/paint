pub mod query;

use crate::store::{self, Stores};
use canvas::resource::*;
use chrono::{DateTime, Utc};
use pigment::{cache, Selector, View};
use query::JsQuery;
pub use query::Query;
use serde::Serialize;
use std::{fmt::Display, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

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
        JsValue::from_serde(
            &self
                .stores
                .query_view(
                    &view.into_serde().map_err(into_exception)?,
                    &query.into_serde::<Query>().map_err(into_exception)?,
                )
                .map_err(into_exception)?,
        )
        .map_err(into_exception)
    }

    /// Update the views in the store.
    pub fn update(&self, view: &JsView, since: &str) -> Result<(), JsValue> {
        let stores = self.stores.clone();
        let view = view.into_serde().map_err(into_exception)?;
        let since = since.parse::<DateTime<Utc>>().map_err(into_exception)?;

        spawn_local(async move {
            store::oil::update_stores(&stores, since, &view)
                .await
                .expect("failed to update stores")
        });
        Ok(())
    }
}

macro_rules! impl_stores_query_view {
    ($( $name:ident : $ty:ty, )*) => {
        impl Stores {
            fn query_view<S>(&self, view: &View, selector: &S) -> cache::Result<QueryResults>
            where
                $( S: Selector<$ty> ),*
            {
                fn collect<T, E>(iter: impl Iterator<Item = Result<T, E>>) -> Result<Vec<T>, E> {
                    let mut vec = Vec::with_capacity(iter.size_hint().0);
                    for item in iter {
                        vec.push(item?);
                    }
                    Ok(vec)
                }

                Ok(QueryResults {
                    $(
                        $name: collect(
                            cache::get_all::<_, $ty>(&self.$name, view)?
                                .map(|res| res.map(|(_, entry)| entry.resource))
                                .filter(|res| res.is_err() || selector.matches(res.as_ref().unwrap())),
                        )?,
                    )*
                })
            }
        }
    };
}
impl_stores_query_view! {
    courses: Course,
    assignments: Assignment,
    submissions: Submission,
}

#[derive(Serialize)]
pub struct QueryResults {
    pub courses: Vec<Course>,
    pub assignments: Vec<Assignment>,
    pub submissions: Vec<Submission>,
}

fn into_exception(err: impl Display) -> JsValue {
    JsValue::from_str(&format!("{}", err))
}

#[wasm_bindgen(typescript_custom_section)]
const TS_VIEW: &str = r#"
export type View = {
    truth: { base_url: string; };
    viewer: { User: number; };
};
"#;

// TODO: add stricter types for resources
#[wasm_bindgen(typescript_custom_section)]
const TS_QUERY_RESULTS: &str = r#"
export type QueryResults = {
    courses: any[];
    assignments: any[];
    submissions: any[];
};
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "View")]
    pub type JsView;
}
