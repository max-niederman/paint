use super::{get_view, DbResource, HttpClient};
use crate::routes::ApiTags;
use crate::{auth::Claims, view::*};
use bson::doc;
use canvas_lms::resource::Course;
use chrono::format::InternalNumeric;
use futures::prelude::*;
use hyper::Method;
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use mongodb::{Collection, Database};
use poem::{
    error::{InternalServerError, NotFoundError},
    Result,
};
use poem_openapi::types::Any;
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
    #[oai(path = "/views/:view_id/courses", method = "get", tag = "ApiTags::Canvas")]
    #[tracing::instrument(skip(self), fields(view_id = ?view_id.0))]
    async fn get_courses(&self, claims: Claims, view_id: Path<Uuid>) -> Result<Json<Vec<Any<Course>>>> {
        claims.ensure_scopes(["read:canvas"])?;

        let view = get_view(&self.views, view_id.0.into()).await?;

        let mut cached_courses: Vec<_> = self.courses
                .find(doc! { "view": view.id }, None)
                .await
                .map_err(InternalServerError)?
                .map_ok(|course| Any(course.resource))
                .try_collect()
                .await
                .map_err(InternalServerError)?;

        if cached_courses.is_empty() {
            let mut courses = view.client(self.http.clone())
                               .request(Method::GET, "/api/v1/courses")
                               .paginate_owned(100)
                               .map_err(InternalServerError)?
                               .items::<Course>()
                               .map_err(InternalServerError);

            while let Some(item) = courses.next().await {
                let course = item?;
                cached_courses.push(Any(course));
                // FIXME: insert into database
            }

        }

        Ok(Json(cached_courses))
    }
}
