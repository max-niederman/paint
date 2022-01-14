use super::Query;
use crate::store::GlazeStore;
use canvas::resource::*;
use pigment::{cache, Selector, View};
use serde::Serialize;
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

    /// Start the query.
    pub fn run_query(&mut self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(
            &self
                .stores
                .query_view(todo!("take view as argument"), &self.query)
                .map_err(|err| JsValue::from_str(&format!("{}", err)))?,
        )
        .map_err(|err| JsValue::from_str(&format!("{}", err)))
    }

    /// Subscribe a callback to the query result.
    pub fn subscribe(&mut self, f: js_sys::Function) {
        self.subscribers.push(f);
    }

    /// Unsubscribe a callback from the query result.
    pub fn unsubscribe(&mut self, f: js_sys::Function) {
        self.subscribers.retain(|x| x != &f);
    }

    #[wasm_bindgen(js_name = "setQueryText")]
    pub fn set_query_text(&mut self, text: Option<String>) {
        self.query.text = text;
    }
}

#[derive(Serialize)]
pub struct QueryResults {
    pub courses: Vec<Course>,
    pub assignments: Vec<Assignment>,
    pub submissions: Vec<Submission>,
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
                            cache::get_all::<_, $ty>(&self.courses, view)?
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
