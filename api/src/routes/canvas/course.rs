use std::time::Duration;

use super::{expiration_index, get_view, DbResource, HttpClient};
use crate::routes::ApiTags;
use crate::{auth::Claims, view::*};
use bson::doc;
use canvas_lms::resource::Course;
use futures::prelude::*;
use hyper::client::HttpConnector;
use hyper::Method;
use hyper_rustls::HttpsConnector;
use miette::{IntoDiagnostic, WrapErr};
use mongodb::{Collection, Database};
use poem::{error::InternalServerError, Result};
use poem_openapi::types::Any;
use poem_openapi::{param::Path, payload::Json, OpenApi};
use tracing::Instrument;
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

    pub async fn init_database(database: &Database) -> miette::Result<()> {
        let collection: Collection<DbResource<Course>> = database.collection("courses");

        collection
            .create_index(
                expiration_index(Duration::from_secs(60 * 60 * 24 * 7)),
                None,
            )
            .await
            .into_diagnostic()
            .wrap_err("failed creating expiration index")?;

        Ok(())
    }
}

#[OpenApi]
impl Api {
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

        let mut cached_courses: Vec<_> = self
            .courses
            .find(doc! { "view": view.id }, None)
            .await
            .map_err(InternalServerError)?
            .map_ok(|course| Any(course.resource))
            .try_collect()
            .await
            .map_err(InternalServerError)?;

        if cached_courses.is_empty() {
            tracing::debug!("cache miss");

            let mut canvas_courses = view
                .client(self.http.clone())
                .request(Method::GET, "/api/v1/courses")
                .paginate_owned(100)
                .map_err(InternalServerError)?
                .items::<Course>()
                .map_err(InternalServerError);

            while let Some(item) = canvas_courses.next().await {
                let course = item?;

                cached_courses.push(Any(course));
            }

            tokio::spawn({
                let course_collection = self.courses.clone();
                let cached_courses = cached_courses.clone();
                let view_id = view.id.into();
                async move {
                    let now = bson::DateTime::now();

                    let res = course_collection
                        .insert_many(
                            cached_courses.into_iter().map(|course| DbResource {
                                view: view_id,
                                inserted_at: now,
                                resource: course.0,
                            }),
                            None,
                        )
                        .await;

                    match res {
                        Ok(_) => tracing::debug!("successfully updated cache"),
                        Err(err) => tracing::error!("failed to update cache: {}", err),
                    }
                }
                .instrument(tracing::info_span!("cache_update"))
            });
        }

        Ok(Json(cached_courses))
    }
}
