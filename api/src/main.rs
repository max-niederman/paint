use poem::{listener::TcpListener, Route};
use poem_openapi::{param::Query, payload::PlainText, OpenApi, OpenApiService};
use tracing_subscriber::EnvFilter;
use miette::{Result, IntoDiagnostic};

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>) -> PlainText<String> {
        match name.0 {
            Some(name) => PlainText(format!("hello, {}!", name)),
            None => PlainText("hello!".to_string()),
        }
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

    let api =
        OpenApiService::new(Api, "Oil", env!("CARGO_PKG_VERSION")).server("http://localhost:4210");
    let app = Route::new().nest("/swagger", api.swagger_ui()).nest("/", api);

    let listen_addr = std::env::var("OIL_ADDR").unwrap_or_else(|_| "0.0.0.0:4210".into());
    tracing::info!(message = "started listening", %listen_addr);
    poem::Server::new(TcpListener::bind(listen_addr))
        .run(app)
        .await
        .into_diagnostic()
}
