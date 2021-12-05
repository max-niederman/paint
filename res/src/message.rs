// memory usage doesn't really matter, so wasting a bit of the stack is alright
#![allow(clippy::large_enum_variant)]

use crate::{View, DSelector};

use canvas_lms::resource;
use serde::{Deserialize, Serialize};

/// A request sent to the server by the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Update {
        /// The view to update.
        view: View,
    },
    Query {
        /// The viewer being queried.
        view: View,
        /// The resource selector.
        selector: DSelector,
    },
}

/// A response sent by the server to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Response {
    UpdateResult {
        downloaded: u32,
        updated: u32,

        canvas_time: f64,
        canvas_cost: f64,
    },
    Resource(DResource),
}

/// A discriminated resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DResource {
    Assignment(resource::Assignment),
    Course(resource::Course),
    Submission(resource::Submission),
}
