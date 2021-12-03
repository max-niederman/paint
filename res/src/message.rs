use crate::{DSelector, Viewer};

use canvas_lms::resource;
use serde::{Deserialize, Serialize};

/// A request sent to the server by the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Update {
        /// The viewer to use for the update.
        viewer: Viewer,
    },
    Query {
        /// The viewer performing the query.
        viewer: Viewer,
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
