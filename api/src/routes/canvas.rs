use super::ApiTags;
use crate::{
    auth::Claims,
    res::{cache, collections},
    view::{DbView, View},
};
use bson::doc;
use canvas_lms::client::hyper::{self, client::HttpConnector};
use hyper_rustls::HttpsConnector;
use mongodb::{Collection, Database};
use poem::{
    error::{InternalServerError, NotFoundError},
    IntoResponse, Response, Result,
};
use poem_openapi::{param::Path, ApiResponse, OpenApi};
use uuid::Uuid;

pub struct Api {
    pub cache: cache::Cache,
    pub views: Collection<DbView>,
    pub http: hyper::Client<HttpsConnector<HttpConnector>>,
}

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

        let view: View = self
            .views
            .find_one(doc! { "_id": view_id.0, "user": claims.sub }, None)
            .await
            .map_err(InternalServerError)?
            .ok_or(NotFoundError)?
            .into();
        
        Ok(self
            .cache
            .cached_fetch(self.http.clone(), &view, &collections::AllCourses)
            .await?
            .into())
    }
}

struct CollectionResponse(sled::Iter);

impl From<sled::Iter> for CollectionResponse {
    fn from(iter: sled::Iter) -> Self {
        Self(iter)
    }
}

impl IntoResponse for CollectionResponse {
    fn into_response(self) -> Response {
        let mut body = b"[".to_vec();

        for entry in self.0 {
            match entry {
                Ok((_key, val)) => {
                    body.extend_from_slice(&val);
                    body.push(b',');
                }
                Err(err) => {
                    return InternalServerError(err).as_response();
                }
            }
        }

        body.pop(); // remove the trailing comma
        body.push(b']');

        Response::builder()
            .content_type("application/json")
            .body(body)
    }
}

impl ApiResponse for CollectionResponse {
    const BAD_REQUEST_HANDLER: bool = false;

    fn meta() -> poem_openapi::registry::MetaResponses {
        use poem_openapi::registry::*;

        MetaResponses {
            responses: vec![MetaResponse {
                description: "a list of Canvas resources",
                status: Some(200),
                content: vec![],
                headers: vec![],
            }],
        }
    }

    fn register(_registry: &mut poem_openapi::registry::Registry) {}
}

struct CollectionResponseSingle(sled::IVec);

impl From<sled::IVec> for CollectionResponseSingle {
    fn from(ivec: sled::IVec) -> Self {
        Self(ivec)
    }
}

impl IntoResponse for CollectionResponseSingle {
    fn into_response(self) -> Response {
        Response::builder()
            .content_type("application/json")
            .body(self.0.to_vec()) // TODO: can we avoid the copy here?
    }
}

impl ApiResponse for CollectionResponseSingle {
    const BAD_REQUEST_HANDLER: bool = false;

    fn meta() -> poem_openapi::registry::MetaResponses {
        use poem_openapi::registry::*;

        MetaResponses {
            responses: vec![MetaResponse {
                description: "a Canvas resource",
                status: Some(200),
                content: vec![],
                headers: vec![],
            }],
        }
    }

    fn register(_registry: &mut poem_openapi::registry::Registry) {}
}
