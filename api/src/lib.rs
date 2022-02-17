#![feature(never_type)]
#![feature(once_cell)]
#![feature(box_patterns)]
#![feature(result_option_inspect)]
#![feature(trivial_bounds)]

use poem_openapi::{OpenApi, Tags, payload::PlainText};

pub mod auth;
pub mod view;

pub struct Api;

#[OpenApi]
impl Api {
    /// Get the API version
    #[oai(path = "/version", method = "get", tag = "ApiTags::Meta")]
    async fn get_version(&self) -> PlainText<&'static str> {
        PlainText(env!("CARGO_PKG_VERSION"))
    }
}

#[derive(Tags)]
enum ApiTags {
    /// Metadata about the API.
    Meta,
    /// A view is a user in a Canvas instance.
    /// Most users will have only one view corresponding to their student account, but some users may have multiple.
    View,
}