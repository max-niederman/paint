#![feature(box_patterns)]

extern crate canvas_lms as canvas;

use miette::{Context, IntoDiagnostic, Result};
use poem::{listener::TcpListener, middleware::Tracing, EndpointExt, Route};
use poem_openapi::{OpenApi, OpenApiService};
use tracing_subscriber::EnvFilter;

struct Api;

#[OpenApi]
impl Api {
    // TODO: add API logic
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
        .with(Tracing);

    let listen_addr = std::env::var("OIL_ADDR").unwrap_or_else(|_| "0.0.0.0:4210".into());
    poem::Server::new(TcpListener::bind(listen_addr))
        .run(app)
        .await
        .into_diagnostic()
}
