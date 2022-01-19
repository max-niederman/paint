// memory usage doesn't really matter here, so wasting a bit of the stack is alright
#![allow(clippy::large_enum_variant)]

use std::str::FromStr;

use canvas::DateTime;
use pigment::View;
use serde::{Deserialize, Serialize};

/// A request sent to the server by the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Fetch {
        /// The view to update.
        view: View,
        /// The Canvas token to use.
        canvas_token: String,
    },
    Update {
        /// The kind of resource.
        resource_kind: ResourceKind,
        /// The viewer being queried.
        view: View,
        /// Date of last update.
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

/// A kind of resource.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResourceKind {
    Assignment,
    Course,
    Submission,
}

impl FromStr for ResourceKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "assignment" => Ok(Self::Assignment),
            "course" => Ok(Self::Course),
            "submission" => Ok(Self::Submission),
            _ => Err("no such resource kind"),
        }
    }
}
