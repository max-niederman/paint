use crate::{auth::Claims, view::DbView};
use bson::doc;
use hyper::{client::connect::Connect, Uri};
use mongodb::{Collection, Database};
use poem::{
    error::{InternalServerError, NotFoundError},
    web::Path,
    Endpoint, FromRequest, Request, Response,
};
use poem_openapi::ApiExtractor;
use serde::Deserialize;
use uuid::Uuid;

pub struct CanvasEndpoint<Conn> {
    http: hyper::Client<Conn>,
    views: Collection<DbView>,
}

impl<Conn> CanvasEndpoint<Conn> {
    pub fn new(database: &Database, http: hyper::Client<Conn>) -> Self {
        Self {
            http,
            views: database.collection("views"),
        }
    }
}

#[poem::async_trait]
impl<Conn> Endpoint for CanvasEndpoint<Conn>
where
    Conn: Clone + Connect + Send + Sync + 'static,
{
    type Output = Response;

    async fn call(&self, req: Request) -> poem::Result<Response> {
        let claims =
            Claims::from_request(&req, &mut Default::default(), Default::default()).await?;
        claims.ensure_scopes(["read:views", "canvas"])?;

        #[derive(Deserialize)]
        struct CanvasPath {
            view_id: Uuid,
            path: String,
        }
        let CanvasPath { view_id, path } = Path::from_request_without_body(&req).await?.0;

        let view = self
            .views
            .find_one(doc! { "_id": view_id, "user": claims.sub }, None)
            .await
            .map_err(InternalServerError)?
            .ok_or(NotFoundError)?;

        let canvas_request = hyper::Request::builder()
            .uri(
                Uri::builder()
                    .scheme("https")
                    .authority(view.canvas_domain)
                    .path_and_query(&path)
                    .build()
                    .map_err(InternalServerError)?, // TODO: better error handling
            )
            .method(req.method())
            .header(
                "Authorization",
                format!("Bearer {}", view.canvas_access_token),
            )
            .body(hyper::Body::wrap_stream(
                req.into_body().into_bytes_stream(),
            ))
            .map_err(InternalServerError)?;

        let mut canvas_response = self
            .http
            .request(canvas_request)
            .await
            .map_err(InternalServerError)?;

        let mut response = Response::builder()
            .status(canvas_response.status())
            .finish();
        
        std::mem::swap(response.extensions_mut(), canvas_response.extensions_mut());
        std::mem::swap(response.headers_mut(), canvas_response.headers_mut());
        response.set_body(canvas_response.into_body());

        Ok(response)
    }
}
