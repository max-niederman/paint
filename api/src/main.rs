#![feature(never_type)]
#![feature(once_cell)]
#![feature(box_patterns)]
#![feature(result_option_inspect)]

extern crate canvas_lms as canvas;

mod auth;

use auth::{update_jwks, Claims};
use futures::prelude::*;
use miette::{IntoDiagnostic, Result};
use poem::{
    listener::TcpListener,
    middleware::{Cors, Tracing},
    EndpointExt, IntoResponse, Route,
};
use poem_openapi::{
    payload::{Json, PlainText},
    OpenApi, OpenApiService,
};
use trace_errors::TraceErrors;
use tracing_subscriber::EnvFilter;
struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/version", method = "get")]
    async fn get_version(&self) -> PlainText<&'static str> {
        PlainText(env!("CARGO_PKG_VERSION"))
    }

    // TODO: add API logic
    #[oai(path = "/instances", method = "get")]
    async fn get_instances(&self, claims: Claims) -> Json<Vec<serde_json::Value>> {
        Json(todo!())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();

    let api = OpenApiService::new(Api, env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .server("http://localhost:4210");

    let app = Route::new()
        .nest("/swagger", api.swagger_ui())
        .nest("/", api)
        .with(TraceErrors)
        .with(Cors::new())
        .with(Tracing);

    let mut update_jwks = Box::pin(auth::update_jwks());
    update_jwks.next().await.unwrap()?; // make sure we have a JWKS to start with
    tokio::spawn(async move {
        loop {
            update_jwks
                .next()
                .await
                .unwrap()
                .expect("update_jwks failed");
        }
    });

    let listen_addr = std::env::var("OIL_ADDR").unwrap_or_else(|_| "0.0.0.0:4210".into());
    poem::Server::new(TcpListener::bind(listen_addr))
        .run(app)
        .await
        .into_diagnostic()
}

mod trace_errors {
    use poem::{Endpoint, Middleware, Request};

    pub struct TraceErrors;

    impl<E: Endpoint> Middleware<E> for TraceErrors {
        type Output = TraceErrorsEndpoint<E>;

        fn transform(&self, ep: E) -> Self::Output {
            TraceErrorsEndpoint(ep)
        }
    }

    pub struct TraceErrorsEndpoint<E>(E);

    #[poem::async_trait]
    impl<E: Endpoint> Endpoint for TraceErrorsEndpoint<E> {
        type Output = E::Output;

        async fn call(&self, req: Request) -> poem::Result<Self::Output> {
            self.0.call(req).await.inspect_err(|err| {
                tracing::error!(%err, "request errored");
            })
        }
    }
}
