use canvas_lms::client::{self, Client};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A Canvas view associated with a user
#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct View {
    /// The unique identifier of the view.
    pub id: Uuid,

    /// The name of the view.
    pub name: String,

    /// The base URL of the Canvas view.
    pub canvas_base_url: String,
    /// The user's Canvas ID.
    pub canvas_user_id: u64,
    /// The user's Canvas access token.
    pub canvas_access_token: String,
}

impl View {
    pub fn client<Conn: Clone>(&self, http: client::hyper::Client<Conn>) -> Client<Conn> {
        Client::<Conn>::builder()
            .base_url(self.canvas_base_url.clone())
            .auth(client::Auth::Bearer(self.canvas_access_token.clone()))
            .build(http)
    }
}
