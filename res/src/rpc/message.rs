// memory usage doesn't really matter, so wasting a bit of the stack is alright
#![allow(clippy::large_enum_variant)]

use canvas_lms::resource;
use pigment::{DSelector, View};
use serde::{Deserialize, Serialize};

/// A request sent to the server by the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Update {
        /// The view to update.
        view: View,
        /// The Canvas token to use.
        canvas_token: String,
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
pub enum Response {
    UpdateFinished,
    UpdateProgress { resource: String },

    Resource(DResource),
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
