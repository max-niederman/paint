pub mod course;

use poem::{error::InternalServerError, IntoResponse, Response};
use poem_openapi::ApiResponse;

#[macro_export]
macro_rules! canvas_api_struct {
    ($vis:vis $name:ident) => {
        $vis struct $name {
            cache: $crate::res::cache::Cache,
            views: mongodb::Collection<$crate::view::DbView>,
            http: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
        }

        impl $name {
            $vis fn new(
                cache: $crate::res::cache::Cache,
                database: &mongodb::Database,
                http: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
            ) -> Self {
                Self {
                    cache,
                    views: database.collection("views"),
                    http,
                }
            }

            async fn get_view(&self, claims: &$crate::auth::Claims, id: uuid::Uuid) -> poem::Result<$crate::view::View> {
                Ok(self.views
                    .find_one(bson::doc! { "_id": id, "user": claims.sub.clone() }, None)
                    .await
                    .map_err(poem::error::InternalServerError)?
                    .ok_or(poem::error::NotFoundError)?
                    .into())
            }
        }
    };
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
