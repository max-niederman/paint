use super::{get_view, DbResource, HttpClient};
use crate::Error;
use crate::{routes::ApiTags, auth::Claims, view::*};
use bson::doc;
use canvas_lms::resource::Course;
use futures::prelude::*;
use hyper::client::HttpConnector;
use hyper::Method;
use hyper_rustls::HttpsConnector;
use mongodb::{Collection, Database};
use poem::error::NotFoundError;
use poem_openapi::types::Any;
use poem_openapi::{param::Path, payload::Json, OpenApi};
use uuid::Uuid;

pub struct Api {
    db_client: mongodb::Client,
    views: Collection<DbView>,
    courses: Collection<DbResource<Course>>,

    http: HttpClient,
}

impl Api {
    pub fn new(
        database: &Database,
        db_client: &mongodb::Client,
        http: hyper::Client<HttpsConnector<HttpConnector>>,
    ) -> Self {
        Self {
            db_client: db_client.clone(),
            views: database.collection("views"),
            courses: database.collection("courses"),
            http,
        }
    }
}

#[OpenApi]
impl Api {
    /// Update the course cache.
    #[oai(
        path = "/views/:view_id/courses/update",
        method = "post",
        tag = "ApiTags::Canvas"
    )]
    #[tracing::instrument(skip(self), fields(view_id = ?view_id.0))]
    async fn update_courses(&self, claims: Claims, view_id: Path<Uuid>) -> poem::Result<()> {
        let view = get_view(&self.views, view_id.0.into()).await?.ok_or(NotFoundError)?;

        let mut session = self
            .db_client
            .start_session(None)
            .await
            .map_err(|err| Error::database_while("creating session for atomic cache update", err))?;

        session
            .start_transaction(None)
            .await
            .map_err(|err| Error::database_while("starting transaction for atomic cache update", err))?;

        self.courses
            .delete_many_with_session(doc! { "view": view.id }, None, &mut session)
            .await
            .map_err(|err| Error::database_while("deleting old cache data", err))?;

        let mut upstream_pages = view
            .client(self.http.clone())
            .request(Method::GET, "/api/v1/courses")
            .extend_include(["favorites"])
            .paginate_owned(100)
            .map_err(|err| Error::canvas_while("creating course pagination stream", err))?
            .and_then(|resp| resp.deserialize::<Vec<Course>>().boxed())
            .map_err(|err| Error::canvas_while("deserializing course response page", err));

        // TODO: it would be slightly better to allow each insertion to run concurrently rather than blocking on each one
        let now = bson::DateTime::now();
        while let Some(page) = upstream_pages.next().await.transpose()? {
            self.courses
                .insert_many_with_session(
                    page.into_iter().map(|course| DbResource {
                        view: view.id,
                        inserted_at: now,
                        resource: course,
                    }),
                    None,
                    &mut session,
                )
                .await
                .map_err(|err| Error::database_while("inserting courses into the cache", err))?;
        }

        session
            .commit_transaction()
            .await
            .map_err(|err| Error::database_while("commiting cache update transaction", err))?;

        Ok(())
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
    ) -> poem::Result<Json<Vec<Any<Course>>>> {
        claims.ensure_scopes(["read:canvas"])?;

        let view = get_view(&self.views, view_id.0.into()).await?.ok_or(NotFoundError)?;

        // TODO: can we avoid the buffering here and start sending immediately?
        let courses: Vec<_> = self
            .courses
            .find(doc! { "view": view.id }, None)
            .await
            .map_err(|err| Error::database_while("creating course cursor", err))?
            .map_ok(|course| course.resource)
            .try_collect()
            .await
            .map_err(|err| Error::database_while("collecting courses into list", err))?;

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
        course_id: Path<i32>,
    ) -> poem::Result<Json<Any<Course>>> {
        claims.ensure_scopes(["read:canvas"])?;

        let view = get_view(&self.views, view_id.0.into()).await?.ok_or(NotFoundError)?;

        let course = self
            .courses
            .find_one(doc! { "view": view.id, "resource.id": course_id.0 }, None)
            .await
            .map_err(|err| Error::database_while("fetching course", err))?
            .ok_or(NotFoundError)?
            .resource;

        Ok(Json(Any(course)))
    }
}
