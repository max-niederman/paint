use crate::{DSelector, Viewer};

use serde::{Serialize, Deserialize};

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
    }
}