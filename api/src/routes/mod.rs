use poem_openapi::{payload::PlainText, OpenApi, Tags};

pub mod canvas;
pub mod view;

pub struct RootApi;

#[OpenApi]
impl RootApi {
    /// Get the API version
    #[oai(path = "/version", method = "get", tag = "ApiTags::Meta")]
    async fn get_version(&self) -> PlainText<&'static str> {
        PlainText(env!("CARGO_PKG_VERSION"))
    }
}

#[derive(Tags)]
pub enum ApiTags {
    /// Metadata about the API.
    Meta,
    /// A view is a user in a Canvas instance.
    /// Most users will have only one view corresponding to their student account, but some users may have multiple.
    View,
    /// Requests allowing a user to interact with a Canvas view.
    Canvas,
}
