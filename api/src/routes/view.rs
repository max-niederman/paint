use super::ApiTags;
use crate::{auth::Claims, view::*, Error};
use bson::doc;
use futures::prelude::*;
use mongodb::{Collection, Database};
use poem::error::NotFoundError;
use poem_openapi::{param::Path, payload::Json, Object, OpenApi};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A new view to be created by the client
#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NewView {
    pub name: String,
    pub canvas_domain: String,
    pub canvas_user_id: u64,
    pub canvas_access_token: String,
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
    async fn get_views(&self, claims: Claims) -> poem::Result<Json<Vec<View>>> {
        claims.ensure_scopes(["read:views"])?;

        Ok(Json(
            self.collection
                .find(doc! { "user": claims.sub.to_string() }, None)
                .await
                .map_err(|err| Error::database_while("creating views cursor", err))?
                .map_ok(View::from)
                .try_collect()
                .await
                .map_err(|err| Error::database_while("collecting views", err))?,
        ))
    }

    /// Get a view by its ID
    #[oai(path = "/views/:id", method = "get", tag = "ApiTags::View")]
    #[tracing::instrument(skip(self), fields(id = ?id.0))]
    async fn get_view(&self, claims: Claims, id: Path<Uuid>) -> poem::Result<Json<View>> {
        claims.ensure_scopes(["read:views"])?;

        Ok(Json(
            self.collection
                .find_one(doc! { "_id": id.0, "user": claims.sub }, None)
                .await
                .map_err(|err| Error::database_while("fetching view", err))?
                .ok_or(NotFoundError)?
                .into(),
        ))
    }

    /// Create a new view
    #[oai(path = "/views", method = "post", tag = "ApiTags::View")]
    #[tracing::instrument(skip(self))]
    async fn post_view(&self, claims: Claims, new_view: Json<NewView>) -> poem::Result<Json<View>> {
        let new_view = new_view.0;

        claims.ensure_scopes(["write:views"])?;

        let db_view = DbView {
            id: Uuid::new_v4().into(),
            name: new_view.name,
            user: claims.sub,
            canvas_domain: new_view.canvas_domain,
            canvas_user_id: new_view.canvas_user_id,
            canvas_access_token: new_view.canvas_access_token,
        };

        self.collection
            .insert_one(&db_view, None)
            .await
            .map_err(|err| Error::database_while("inserting view", err))?;

        Ok(Json(db_view.into()))
    }

    /// Delete a view
    #[oai(path = "/views/:id", method = "delete", tag = "ApiTags::View")]
    #[tracing::instrument(skip(self), fields(id = ?id.0))]
    async fn delete_view(&self, claims: Claims, id: Path<Uuid>) -> poem::Result<()> {
        claims.ensure_scopes(["write:views"])?;

        self.collection
            .delete_one(doc! { "_id": id.0, "user": claims.sub }, None)
            .await
            .map_err(|err| Error::database_while("deleting view", err))?;

        Ok(())
    }
}
