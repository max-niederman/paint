use super::{get_view, DbResource, HttpClient};
use crate::routes::ApiTags;
use crate::{auth::Claims, view::*};
use bson::doc;
use canvas_lms::resource::Course;
use futures::prelude::*;
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use mongodb::{Collection, Database};
use poem::{
    error::{InternalServerError, NotFoundError},
    Result,
};
use poem_openapi::{param::Path, payload::Json, Object, OpenApi};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct Api {
    views: Collection<DbView>,
    courses: Collection<DbResource<Course>>,

    http: HttpClient,
}

impl Api {
    pub fn new(database: &Database, http: hyper::Client<HttpsConnector<HttpConnector>>) -> Self {
        Self {
            views: database.collection("views"),
            courses: database.collection("courses"),
            http,
        }
    }
}

#[OpenApi]
impl Api {
    /// Get all courses.
    #[oai(path = "/courses", method = "get", tag = "ApiTags::Canvas")]
    #[tracing::instrument(skip(self), fields(view_id = ?view_id.0))]
    async fn get_courses(&self, claims: Claims, view_id: Path<Uuid>) -> Result<Json<Vec<Course>>> {
        claims.ensure_scopes(["read:canvas"])?;

        let view = get_view(&self.views, view_id.0.into()).await?;

        Ok(Json(
            self.courses
                .find(doc! { "view": view.id }, None)
                .await
                .map_err(InternalServerError)?
                .map_ok(|course| course.resource)
                .try_collect()
                .await
                .map_err(InternalServerError)?,
        ))
    }
}
