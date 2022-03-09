use futures::prelude::*;
use hyper_rustls::HttpsConnectorBuilder;
use miette::{IntoDiagnostic, WrapErr};
use oil::{auth, routes};
use poem::{
    listener::TcpListener,
    middleware::{Cors, Tracing},
    EndpointExt, Route,
};
use poem_openapi::OpenApiService;
use std::time::Duration;
use tokio::task;

// TODO: send proper, consistent error responses for all error types

#[tokio::main]
async fn main() -> miette::Result<()> {
    init_logging();

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
    .server("http://localhost:4200");

    let canvas_api = routes::canvas::CanvasEndpoint::new(
        &database,
        hyper::Client::builder().build(
            HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
    );

    let app = Route::new()
        .nest("/docs", api.rapidoc())
        .nest("/", api)
        .at("/canvas/:view_id/*path", canvas_api)
        .with(Cors::new())
        .with(Tracing);

    tracing::info!("starting JWK update task");
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
    tracing::info!(%listen_addr, "starting web server");
    poem::Server::new(TcpListener::bind(listen_addr))
        .run(app)
        .await
        .into_diagnostic()
}

fn init_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
    use tracing_tree::HierarchicalLayer;

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .init();
}
