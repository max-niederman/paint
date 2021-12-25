// memory usage doesn't really matter, so wasting a bit of the stack is alright
#![allow(clippy::large_enum_variant)]

use canvas::{resource, DateTime};
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
        /// The viewer being queried.
        view: View,
        /// Date of last update.
        since: DateTime,
    },
}

/// A response sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Fetch(FetchResponse),
    Update(UpdateResponse),
}

/// A fetch response sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchResponse {
    Progress { resource: String },
}

/// An update response sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateResponse {
    /// A resource which the client does not have.
    Resource(DResource),
    /// A stub standing in for a resource the client has an updated copy of already.
    Stub(Vec<u8>),
}

/// A discriminated resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DResource {
    Assignment(resource::Assignment),
    Course(resource::Course),
    Submission(resource::Submission),
}

macro_rules! impl_dresource_from_resource {
    ($res:ident) => {
        impl From<::canvas_lms::resource::$res> for DResource {
            fn from(r: ::canvas_lms::resource::$res) -> Self {
                Self::$res(r)
            }
        }
    };
    ($($res:ident),* $(,)?) => {
        $(impl_dresource_from_resource!($res);)*
    };
}
impl_dresource_from_resource!(Assignment, Course, Submission);
