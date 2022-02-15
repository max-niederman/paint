use crate::{auth::Claims, ApiTags};
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

/// A Canvas instance associated with a user.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct Instance {
    /// The unique identifier of the instance.
    pub id: Uuid,

    /// The base URL of the Canvas instance.
    pub canvas_base_url: String,
    /// The user's Canvas ID.
    pub canvas_user_id: u64,
    /// The user's Canvas access token.
    pub canvas_access_token: String,
}

/// A new instance to be created by the client.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NewInstance {
    pub canvas_base_url: String,
    pub canvas_user_id: u64,
    pub canvas_access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbInstance {
    #[serde(rename = "_id")]
    id: bson::Uuid,
    user: String,
    canvas_view: pigment::View,
    canvas_access_token: String,
}

impl From<DbInstance> for Instance {
    fn from(db_instance: DbInstance) -> Self {
        Instance {
            id: db_instance.id.into(),
            canvas_base_url: db_instance.canvas_view.truth.base_url,
            canvas_user_id: match db_instance.canvas_view.viewer {
                pigment::view::Viewer::User(id) => id.into(),
            },
            canvas_access_token: db_instance.canvas_access_token,
        }
    }
}

pub struct Api {
    collection: Collection<DbInstance>,
}

impl Api {
    pub fn new(database: &Database) -> Self {
        Api {
            collection: database.collection("instances"),
        }
    }
}

#[OpenApi]
impl Api {
    #[oai(path = "/instances", method = "get", tag = "ApiTags::Instance")]
    #[tracing::instrument(skip(self))]
    async fn get_instances(&self, claims: Claims) -> Result<Json<Vec<Instance>>> {
        claims.ensure_scopes(["read:instances"])?;

        Ok(Json(
            self.collection
                .find(doc! { "user": claims.sub.to_string() }, None)
                .await
                .map_err(InternalServerError)?
                .map_ok(Instance::from)
                .try_collect()
                .await
                .map_err(InternalServerError)?,
        ))
    }

    #[oai(path = "/instances/:id", method = "get", tag = "ApiTags::Instance")]
    #[tracing::instrument(skip(self), fields(id = ?id.0))]
    async fn get_instance(&self, claims: Claims, id: Path<Uuid>) -> Result<Json<Instance>> {
        claims.ensure_scopes(["read:instances"])?;

        Ok(Json(
            self.collection
                .find_one(doc! { "_id": id.0, "user": claims.sub }, None)
                .await
                .map_err(InternalServerError)?
                .ok_or(NotFoundError)?
                .into(),
        ))
    }

    #[oai(path = "/instances", method = "post", tag = "ApiTags::Instance")]
    #[tracing::instrument(skip(self))]
    async fn post_instance(
        &self,
        claims: Claims,
        new_instance: Json<NewInstance>,
    ) -> Result<Json<Instance>> {
        let new_instance = new_instance.0;

        claims.ensure_scopes(["write:instances"])?;

        let db_instance = DbInstance {
            id: Uuid::new_v4().into(),
            user: claims.sub,
            canvas_view: pigment::View {
                truth: pigment::view::Canvas {
                    base_url: new_instance.canvas_base_url,
                },
                viewer: pigment::view::Viewer::User(new_instance.canvas_user_id.into()),
            },
            canvas_access_token: new_instance.canvas_access_token,
        };

        self.collection
            .insert_one(&db_instance, None)
            .await
            .map_err(InternalServerError)?;

        Ok(Json(db_instance.into()))
    }

    #[oai(path = "/instances/:id", method = "delete", tag = "ApiTags::Instance")]
    #[tracing::instrument(skip(self), fields(id = ?id.0))]
    async fn delete_instance(&self, claims: Claims, id: Path<Uuid>) -> Result<()> {
        claims.ensure_scopes(["write:instances"])?;

        self.collection
            .delete_one(doc! { "_id": id.0, "user": claims.sub }, None)
            .await
            .map_err(InternalServerError)?;

        Ok(())
    }
}
