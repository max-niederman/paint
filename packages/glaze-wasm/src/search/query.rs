use crate::store::{GlazeStore, Stores};
use canvas::Resource;
use miette::{Context, IntoDiagnostic, Result};
use pigment::Selector;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Query {
    pub(crate) count: Option<usize>,
    pub(crate) sorted: bool,

    pub(crate) text: Option<String>,
}

impl Query {
    pub fn execute(&self, stores: &Stores) -> Result<Vec<QueryResult>> {
        let mut results = Vec::with_capacity(self.count.unwrap_or(0));

        self.execute_individual::<canvas::resource::Submission>(&mut results, &stores.submissions)?;
        self.execute_individual::<canvas::resource::Course>(&mut results, &stores.courses)?;
        self.execute_individual::<canvas::resource::Assignment>(&mut results, &stores.assignments)?;

        // results are sorted by score in ascending order, so we reverse them
        results.reverse();

        Ok(results)
    }

    fn execute_individual<R>(
        &self,
        results: &mut Vec<QueryResult>,
        store: &GlazeStore,
    ) -> Result<()>
    where
        R: Resource + Into<QueryResultResource>,
        Self: Score<R>,
    {
        // EXPL: we iterate over each of the store's resources.
        //       each iteration, we check if the resource has a score greater than or equal to
        //       the worst score in `resources`. if it does, we add it to `resources` and sort it.
        //       this sort _should_ actually be O(n) due to the implementation of `slice::sort_unstable`.
        // TODO: ensure that this is actually the fastest way to do this, and figure out if it even matters

        debug_assert!(results.is_sorted_by_key(|r| r.score));

        if self.sorted {
            for resource in store
                .resources
                .iter()
                .map(|entry| bincode::deserialize::<R>(&entry.value()))
            {
                let resource = resource
                    .into_diagnostic()
                    .wrap_err("failed to deserialize resource")?;

                let result = QueryResult {
                    score: self.score(&resource),
                    resource: resource.into(),
                };

                let worst_score = results.first().map(|r| r.score).unwrap_or(isize::MIN);
                if result.score >= worst_score {
                    if results.len() == self.count.unwrap_or(usize::MAX) {
                        results.remove(0);
                    }
                    results.push(result);
                    results.sort_unstable_by_key(|r| r.score);
                }
            }
        } else if results.len() < self.count.unwrap_or(usize::MAX) {
            for resource in store
                .resources
                .iter()
                .rev() // reversed to prioritize resources with higher IDs
                .take(self.count.unwrap_or(usize::MAX) - results.len())
                .map(|entry| bincode::deserialize::<R>(&entry.value()))
            {
                let resource = resource
                    .into_diagnostic()
                    .wrap_err("failed to deserialize resource")?;

                let result = QueryResult {
                    score: self.score(&resource),
                    resource: resource.into(),
                };

                results.push(result);
            }
        }

        Ok(())
    }
}

impl<R: Resource> Selector<R> for Query {
    fn matches(&self, _resource: &R) -> bool {
        // FIXME: implement query matching
        true
    }
}

pub trait Score<R: Resource> {
    /// Score a resource.
    fn score(&self, resource: &R) -> isize;
}

impl<R: Resource> Score<R> for Query {
    fn score(&self, _resource: &R) -> isize {
        // FIXME: implement query ordering
        0
    }
}

macro_rules! query_result_resource_enum {
    ( $( $resource:ident ),* $(,)? ) => {
        #[derive(Debug, Clone, PartialEq, Serialize)]
        #[serde(tag = "type", content = "resource")]
        pub enum QueryResultResource {
            $(
                $resource(canvas::resource::$resource),
            )*
        }

        $(
            impl From<canvas::resource::$resource> for QueryResultResource {
                fn from(resource: canvas::resource::$resource) -> Self {
                    QueryResultResource::$resource(resource)
                }
            }
        )*
    };
}

query_result_resource_enum!(Assignment, Course, Submission);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QueryResult {
    score: isize,
    #[serde(flatten)]
    resource: QueryResultResource,
}

#[wasm_bindgen(typescript_custom_section)]
const TS_QUERY: &str = r#"
export type Query = {
    count?: number;
    sorted: boolean;

    text?: string;
};
"#;

// TODO: add stricter types for resources
#[wasm_bindgen(typescript_custom_section)]
const TS_QUERY_RESULTS: &str = r#"
export type QueryResultResource = 
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

export type QueryResult = 
    {
        score: number;
    } 
    & QueryResultResource;

export type QueryResults = QueryResult[];
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Query")]
    pub type JsQuery;
}
