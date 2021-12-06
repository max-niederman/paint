extern crate canvas_lms as canvas;

mod fetch;

use miette::{IntoDiagnostic, Result};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let listen_addr = std::env::var("PIGMENT_ADDR").unwrap_or_else(|_| "0.0.0.0:4211".into());
    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .into_diagnostic()?;

    loop {
        let (socket, remote) = listener.accept().await.into_diagnostic()?;
        tokio::spawn(async move {
            log::debug!("accepted connection from {}", remote);
        });
    }
}
