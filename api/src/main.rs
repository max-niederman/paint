#![feature(never_type)]
#![feature(once_cell)]
#![feature(box_patterns)]
#![feature(result_option_inspect)]
#![feature(trivial_bounds)]

extern crate canvas_lms as canvas;

pub mod auth;
pub mod view;

use futures::prelude::*;
use miette::{IntoDiagnostic, WrapErr};
use poem::{
    listener::TcpListener,
    middleware::{Cors, Tracing},
    EndpointExt, Route,
};
use poem_openapi::{payload::PlainText, OpenApi, OpenApiService, Tags};
use tracing_subscriber::EnvFilter;
struct Api;

#[OpenApi]
impl Api {
    /// Get the API version
    #[oai(path = "/version", method = "get", tag = "ApiTags::Meta")]
    async fn get_version(&self) -> PlainText<&'static str> {
        PlainText(env!("CARGO_PKG_VERSION"))
    }
}

#[derive(Tags)]
enum ApiTags {
    /// Metadata about the API.
    Meta,
    /// A view is a user in a Canvas instance.
    /// Most users will have only one view corresponding to their student account, but some users may have multiple.
    View,
}

// TODO: send proper, consistent error responses for all error types

#[tokio::main]
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
        (Api, view::Api::new(&database)),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .server("http://localhost:4210");

    let app = Route::new()
        .nest("/docs", api.rapidoc())
        .nest("/", api)
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
