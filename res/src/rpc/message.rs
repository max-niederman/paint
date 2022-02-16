// memory usage doesn't really matter here, so wasting a bit of the stack is alright
#![allow(clippy::large_enum_variant)]

use canvas::DateTime;
use pigment::{DSelector, ResourceKind, View};
use serde::{Deserialize, Serialize};

/// A request sent to the server by the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    /// Fetch upstream view.
    FetchUpstream {
        /// The view to update.
        view: View,
        /// The Canvas token to use.
        canvas_token: String,
    },
    /// Run a query on the view and return the results.
    Query {
        /// The viewer being queried.
        view: View,
        /// The kind of resource.
        resource_kind: ResourceKind,
        /// The selector to match.
        selector: DSelector,
    },
    /// Get a diff between the server's view and the client's view.
    Diff {
        /// The viewer being queried.
        view: View,
        /// The kind of resource.
        resource_kind: ResourceKind,
        /// The selector to match.
        since: DateTime,
    },
}

// TODO: abolish the [`Response`] type entirely and rely on deserialization errors
//       this will require lots of refactoring and I think it's best to wait until
//       the codebase is more stable

/// A response sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Fetch(FetchResponse),
    Update(UpdateResponse),
    Query(QueryResponse),
}

/// A fetch response sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchResponse {
    Progress { resource_kind: ResourceKind },
}

/// An update response sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResponse {
    /// The key of the resource.
    pub key: Vec<u8>,
    /// Serialized [`CacheEntry`] containing the resource.
    /// If the client has an up-to-date copy of the resource, this will be `None`.
    pub resource: Option<Vec<u8>>,
}

/// A query response sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {}
