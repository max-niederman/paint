extern crate canvas_lms as canvas;

use futures::prelude::*;
use miette::{IntoDiagnostic, WrapErr};
use oil::{routes, auth};
use async_std::task;
use poem::{
    listener::TcpListener,
    middleware::{Cors, Tracing},
    EndpointExt, Route,
};
use poem_openapi::OpenApiService;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

// TODO: send proper, consistent error responses for all error types

#[async_std::main]
async fn main() -> miette::Result<()> {
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

    tracing::info!("connecting to MongoDB");
    let mongo_client = mongodb::Client::with_uri_str(
        std::env::var("OIL_MONGODB_URI")
            .into_diagnostic()
            .wrap_err("missing OIL_MONGODB_URI environment variable")?,
    )
    .await
    .into_diagnostic()
    .wrap_err("failed to create MongoDB client")?;
    let database = mongo_client.database("oil");

    let api = OpenApiService::new(
        (routes::RootApi, routes::view::Api::new(&database)),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .server("http://localhost:4210");

    let app = Route::new()
        .nest("/docs", api.rapidoc())
        .nest("/", api)
        .with(Cors::new())
        .with(Tracing);

    let mut update_jwks = auth::update_jwks(
        std::env::var_os("OIL_JWKS_UPDATE_INTERVAL")
            .as_ref()
            .and_then(|s| s.to_str())
            .and_then(|s| s.parse::<f64>().ok())
            .map(Duration::from_secs_f64)
            .unwrap_or_else(|| Duration::from_secs(5 * 60)),
    )
    .boxed();
    update_jwks.next().await.unwrap()?; // make sure we have a JWKS to start with
    task::spawn(async move {
        loop {
            update_jwks
                .next()
                .await
                .unwrap()
                .expect("update_jwks failed");
        }
    });

    let listen_addr = std::env::var("OIL_ADDR").unwrap_or_else(|_| "0.0.0.0:4200".into());
    poem::Server::new(TcpListener::bind(listen_addr))
        .run(app)
        .await
        .into_diagnostic()
}
