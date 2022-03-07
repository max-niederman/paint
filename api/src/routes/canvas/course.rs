use super::{CollectionResponse, CollectionResponseSingle};
use crate::{auth::Claims, canvas_api_struct, res::collections, routes::ApiTags};
use poem::{error::NotFoundError, Result};
use poem_openapi::{
    param::Path,
    OpenApi,
};
use uuid::Uuid;

canvas_api_struct!(pub Api);

#[OpenApi]
impl Api {
    #[oai(
        path = "/views/:view_id/courses",
        method = "get",
        tag = "ApiTags::Canvas"
    )]
    #[tracing::instrument(skip(self), fields(view_id = ?view_id.0))]
    async fn get_all_courses(
        &self,
        claims: Claims,
        view_id: Path<Uuid>,
    ) -> Result<CollectionResponse> {
        claims.ensure_scopes(["read:views", "read:canvas"])?;

        Ok(self
            .cache
            .cached_fetch(
                self.http.clone(),
                &self.get_view(&claims, *view_id).await?,
                &collections::AllCourses,
            )
            .await?
            .into())
    }

    #[oai(
        path = "/views/:view_id/courses/:course_id",
        method = "get",
        tag = "ApiTags::Canvas"
    )]
    #[tracing::instrument(skip(self), fields(view_id = ?view_id.0, course_id = ?course_id.0))]
    async fn get_course(
        &self,
        claims: Claims,
        view_id: Path<Uuid>,
        course_id: Path<u64>,
    ) -> Result<CollectionResponseSingle> {
        claims.ensure_scopes(["read:views", "read:canvas"])?;

        Ok(self
            .cache
            .cached_fetch_one(
                self.http.clone(),
                &self.get_view(&claims, *view_id).await?,
                &collections::CourseById(course_id.0.into()),
            )
            .await?
            .ok_or(NotFoundError)?
            .into())
    }
}
