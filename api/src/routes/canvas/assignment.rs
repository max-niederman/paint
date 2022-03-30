use super::{get_view, DbResource};
use crate::{Error, HttpClient, auth::Claims, routes::ApiTags, view::*};
use bson::doc;
use canvas_lms::resource::Assignment;
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
    assignments: Collection<DbResource<Assignment>>,

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
            assignments: database.collection("assignments"),
            http,
        }
    }
}

#[OpenApi]
impl Api {
    /// Update the assignment cache for a given course.
    #[oai(
        path = "/views/:view_id/courses/:course_id/assignments/update",
        method = "post",
        tag = "ApiTags::Canvas"
    )]
    #[tracing::instrument(skip(self), fields(view_id = ?view_id.0, course_id = ?course_id.0))]
    async fn update_assignments(
        &self,
        claims: Claims,
        view_id: Path<Uuid>,
        course_id: Path<i64>,
    ) -> poem::Result<()> {
        let view = get_view(&self.views, view_id.0.into())
            .await?
            .ok_or(NotFoundError)?;

        let mut session = self.db_client.start_session(None).await.map_err(|err| {
            Error::database_while("creating session for atomic cache update", err)
        })?;

        session.start_transaction(None).await.map_err(|err| {
            Error::database_while("starting transaction for atomic cache update", err)
        })?;

        self.assignments
            .delete_many_with_session(doc! { "view": view.id, "resource.course_id": course_id.0 }, None, &mut session)
            .await
            .map_err(|err| Error::database_while("deleting old cache data", err))?;
        
        let mut upstream_pages = view
            .client(self.http.clone())
            .request(
                Method::GET,
                format!("/api/v1/courses/{}/assignments", course_id.0),
            )
            .extend_include(["submission", "score_statistics"])
            .paginate_owned(100)
            .map_err(|err| Error::canvas_while("creating assignment pagination stream", err))?
            .and_then(|resp| resp.deserialize::<Vec<Assignment>>().boxed())
            .map_err(|err| Error::canvas_while("deserializing assignment response page", err));

        // TODO: it would be slightly better to allow each insertion to run concurrently rather than blocking on each one
        let now = bson::DateTime::now();
        while let Some(page) = upstream_pages.next().await.transpose()? {
            self.assignments
                .insert_many_with_session(
                    page.into_iter().map(|resource| DbResource {
                        view: view.id,
                        inserted_at: now,
                        resource,
                    }),
                    None,
                    &mut session,
                )
                .await
                .map_err(|err| {
                    Error::database_while("inserting assignments into the cache", err)
                })?;
        }

        session
            .commit_transaction()
            .await
            .map_err(|err| Error::database_while("commiting cache update transaction", err))?;

        Ok(())
    }

    /// Get all assignments for a course.
    #[oai(
        path = "/views/:view_id/courses/:course_id/assignments",
        method = "get",
        tag = "ApiTags::Canvas"
    )]
    #[tracing::instrument(skip(self), fields(view_id = ?view_id.0, course_id = ?course_id.0))]
    async fn get_assignments(
        &self,
        claims: Claims,
        view_id: Path<Uuid>,
        course_id: Path<i64>,
    ) -> poem::Result<Json<Vec<Any<Assignment>>>> {
        claims.ensure_scopes(["read:canvas"])?;

        let view = get_view(&self.views, view_id.0.into())
            .await?
            .ok_or(NotFoundError)?;

        // TODO: can we avoid the buffering here and start sending immediately?
        let courses: Vec<_> = self
            .assignments
            .find(doc! { "resource.course_id": course_id.0 }, None)
            // .find(doc! { "view": view.id, "resource.course_id": course_id.0 }, None)
            .await
            .map_err(|err| Error::database_while("creating assignment cursor", err))?
            .map_ok(|course| course.resource)
            .try_collect()
            .await
            .map_err(|err| Error::database_while("collecting assignments into list", err))?;
        
        tracing::debug!("found {} assignments", courses.len());

        Ok(Json(courses.into_iter().map(Any).collect()))
    }

    /// Get a course by its ID.
    #[oai(
        path = "/views/:view_id/courses/:course_id/assignments/:assignment_id",
        method = "get",
        tag = "ApiTags::Canvas"
    )]
    #[tracing::instrument(skip(self), fields(view_id = ?view_id.0, course_id = ?course_id.0, assignment_id = ?assignment_id.0))]
    async fn get_assignment_by_id(
        &self,
        claims: Claims,
        view_id: Path<Uuid>,
        course_id: Path<i64>,
        assignment_id: Path<i64>,
    ) -> poem::Result<Json<Any<Assignment>>> {
        claims.ensure_scopes(["read:canvas"])?;

        let view = get_view(&self.views, view_id.0.into())
            .await?
            .ok_or(NotFoundError)?;

        let assignment = self
            .assignments
            .find_one(doc! { "view": view.id, "resource.course_id": course_id.0, "resource.id": assignment_id.0 }, None)
            .await
            .map_err(|err| Error::database_while("fetching course", err))?
            .ok_or(NotFoundError)?
            .resource;

        Ok(Json(Any(assignment)))
    }
}
