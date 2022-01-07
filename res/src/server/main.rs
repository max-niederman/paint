#![feature(box_patterns)]
#![feature(async_closure)]
#![feature(try_trait_v2)]
#![feature(generic_associated_types)]

extern crate canvas_lms as canvas;

mod cache;
mod handler;
mod store;

use async_bincode::AsyncBincodeStream;
use ebauche::rpc;
use futures::{future, StreamExt};
use handler::Handler;
use miette::{IntoDiagnostic, Result, WrapErr};
use tracing::Instrument;
use tracing_subscriber::EnvFilter;

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

    let listen_addr = std::env::var("PIGMENT_ADDR").unwrap_or_else(|_| "0.0.0.0:4211".into());
    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .into_diagnostic()
        .wrap_err("failed to listen")?;

    let server: &'static _ = Box::leak(Box::new(rpc::Server::new(Handler::new(
        sled::open(std::env::var("EBAUCHE_DB").unwrap_or_else(|_| "db".into()))
            .into_diagnostic()
            .wrap_err("failed to open sled database")?,
    ))));

    tracing::info!(message = "started listening", %listen_addr);
    loop {
        let (socket, peer_addr) = listener.accept().await.into_diagnostic()?;
        tokio::spawn(
            async move {
                tracing::debug!("accepted connection");

                let mut transport = AsyncBincodeStream::from(socket)
                    .for_async()
                    // gracefully handle reset connection
                    .take_while(|e| {
                        future::ready(match e {
                            Err(box bincode::ErrorKind::Io(e)) => {
                                e.kind() == std::io::ErrorKind::ConnectionReset
                            }
                            _ => true,
                        })
                    });

                match server.handle(&mut transport).await {
                    Ok(()) => tracing::debug!("finished handling connection"),
                    Err(ref error) => {
                        tracing::error!(message = "fatal error handling connection", %error)
                    }
                }
            }
            .instrument(tracing::info_span!(
                "connection",
                %peer_addr,
            )),
        );
    }
}
