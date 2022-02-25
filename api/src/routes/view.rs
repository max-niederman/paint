use crate::auth::Claims;
use super::ApiTags;
use bson::doc;
use futures::prelude::*;
use mongodb::{Collection, Database};
use poem::{
    error::{InternalServerError, NotFoundError},
    Result,
};
use poem_openapi::{param::Path, payload::Json, Object, OpenApi};
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

/// A new view to be created by the client
#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NewView {
    pub name: String,
    pub canvas_base_url: String,
    pub canvas_user_id: u64,
    pub canvas_access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbView {
    #[serde(rename = "_id")]
    id: bson::Uuid,
    name: String,
    user: String,
    canvas_base_url: String,
    canvas_user_id: u64,
    canvas_access_token: String,
}

impl From<DbView> for View {
    fn from(db_view: DbView) -> Self {
        View {
            id: db_view.id.into(),
            name: db_view.name,
            canvas_base_url: db_view.canvas_base_url,
            canvas_user_id: db_view.canvas_user_id,
            canvas_access_token: db_view.canvas_access_token,
        }
    }
}

pub struct Api {
    collection: Collection<DbView>,
}

impl Api {
    pub fn new(database: &Database) -> Self {
        Api {
            collection: database.collection("views"),
        }
    }
}

#[OpenApi]
impl Api {
    /// Get all views
    #[oai(path = "/views", method = "get", tag = "ApiTags::View")]
    #[tracing::instrument(skip(self))]
    async fn get_views(&self, claims: Claims) -> Result<Json<Vec<View>>> {
        claims.ensure_scopes(["read:views"])?;

        Ok(Json(
            self.collection
                .find(doc! { "user": claims.sub.to_string() }, None)
                .await
                .map_err(InternalServerError)?
                .map_ok(View::from)
                .try_collect()
                .await
                .map_err(InternalServerError)?,
        ))
    }

    /// Get a view by its ID
    #[oai(path = "/views/:id", method = "get", tag = "ApiTags::View")]
    #[tracing::instrument(skip(self), fields(id = ?id.0))]
    async fn get_view(&self, claims: Claims, id: Path<Uuid>) -> Result<Json<View>> {
        claims.ensure_scopes(["read:views"])?;

        Ok(Json(
            self.collection
                .find_one(doc! { "_id": id.0, "user": claims.sub }, None)
                .await
                .map_err(InternalServerError)?
                .ok_or(NotFoundError)?
                .into(),
        ))
    }

    /// Create a new view
    #[oai(path = "/views", method = "post", tag = "ApiTags::View")]
    #[tracing::instrument(skip(self))]
    async fn post_view(&self, claims: Claims, new_view: Json<NewView>) -> Result<Json<View>> {
        let new_view = new_view.0;

        claims.ensure_scopes(["write:views"])?;

        let db_view = DbView {
            id: Uuid::new_v4().into(),
            name: new_view.name,
            user: claims.sub,
            canvas_base_url: new_view.canvas_base_url,
            canvas_user_id: new_view.canvas_user_id,
            canvas_access_token: new_view.canvas_access_token,
        };

        self.collection
            .insert_one(&db_view, None)
            .await
            .map_err(InternalServerError)?;

        Ok(Json(db_view.into()))
    }

    /// Delete a view
    #[oai(path = "/views/:id", method = "delete", tag = "ApiTags::View")]
    #[tracing::instrument(skip(self), fields(id = ?id.0))]
    async fn delete_view(&self, claims: Claims, id: Path<Uuid>) -> Result<()> {
        claims.ensure_scopes(["write:views"])?;

        self.collection
            .delete_one(doc! { "_id": id.0, "user": claims.sub }, None)
            .await
            .map_err(InternalServerError)?;

        Ok(())
    }
}
