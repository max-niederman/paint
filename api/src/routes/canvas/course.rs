use super::{get_view, DbResource, HttpClient};
use crate::routes::ApiTags;
use crate::{auth::Claims, view::*};
use bson::doc;
use canvas_lms::resource::Course;
use futures::prelude::*;
use hyper::client::HttpConnector;
use hyper::Method;
use hyper_rustls::HttpsConnector;
use mongodb::{Collection, Database};
use poem::error::NotFoundError;
use poem::{error::InternalServerError, Result};
use poem_openapi::types::Any;
use poem_openapi::{param::Path, payload::Json, OpenApi};
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
    /// Update the cache of courses.
    #[oai(
        path = "/views/:view_id/courses/update",
        method = "post",
        tag = "ApiTags::Canvas"
    )]
    #[tracing::instrument(skip(self), fields(view_id = ?view_id.0))]
    async fn update_courses(&self, claims: Claims, view_id: Path<Uuid>) -> Result<()> {
        todo!()
    }

    /// Get all courses.
    #[oai(
        path = "/views/:view_id/courses",
        method = "get",
        tag = "ApiTags::Canvas"
    )]
    #[tracing::instrument(skip(self), fields(view_id = ?view_id.0))]
    async fn get_courses(
        &self,
        claims: Claims,
        view_id: Path<Uuid>,
    ) -> Result<Json<Vec<Any<Course>>>> {
        claims.ensure_scopes(["read:canvas"])?;

        let view = get_view(&self.views, view_id.0.into()).await?;

        // TODO: can we avoid the buffering here and start sending immediately?
        let courses: Vec<_> = self
            .courses
            .find(doc! { "view": view.id }, None)
            .await
            .map_err(InternalServerError)?
            .map_ok(|course| course.resource)
            .try_collect()
            .await
            .map_err(InternalServerError)?;

        Ok(Json(courses.into_iter().map(Any).collect()))
    }

    /// Get a course by its ID.
    #[oai(
        path = "/views/:view_id/courses/:course_id",
        method = "get",
        tag = "ApiTags::Canvas"
    )]
    #[tracing::instrument(skip(self), fields(view_id = ?view_id.0, course_id = ?course_id.0))]
    async fn get_course_by_id(
        &self,
        claims: Claims,
        view_id: Path<Uuid>,
        course_id: Path<u32>,
    ) -> Result<Json<Any<Course>>> {
        claims.ensure_scopes(["read:canvas"])?;

        let view = get_view(&self.views, view_id.0.into()).await?;

        let course = self
            .courses
            .find_one(
                doc! { "view": view.id, "resource": { "id": course_id.0 } },
                None,
            )
            .await
            .map_err(InternalServerError)?
            .ok_or(NotFoundError)?
            .resource;

        Ok(Json(Any(course)))
    }
}
