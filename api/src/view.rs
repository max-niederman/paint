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

    /// The domain of the Canvas instance.
    pub canvas_domain: String,
    /// The user's Canvas ID.
    pub canvas_user_id: u64,
    // /// The user's Canvas access token.
    // pub canvas_access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbView {
    #[serde(rename = "_id")]
    pub id: bson::Uuid,
    pub name: String,
    pub user: String,
    pub canvas_domain: String,
    pub canvas_user_id: u64,
    pub canvas_access_token: String,
}

impl DbView {
    pub fn client<Conn: Clone>(&self, http: hyper::Client<Conn>) -> canvas_lms::Client<Conn> {
        canvas_lms::Client::<Conn>::builder()
            .base_url(format!("https://{}", self.canvas_domain))
            .auth(canvas_lms::client::Auth::Bearer(
                self.canvas_access_token.clone(),
            ))
            .build(http)
    }
}

impl From<DbView> for View {
    fn from(db_view: DbView) -> Self {
        View {
            id: db_view.id.into(),
            name: db_view.name,
            canvas_domain: db_view.canvas_domain,
            canvas_user_id: db_view.canvas_user_id,
            // canvas_access_token: db_view.canvas_access_token,
        }
    }
}
