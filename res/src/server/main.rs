#![feature(box_patterns)]
#![feature(async_closure)]

extern crate canvas_lms as canvas;

mod handler;
mod store;

use async_bincode::AsyncBincodeStream;
use futures::{future, StreamExt};
use handler::Handler;
use miette::{IntoDiagnostic, Result, WrapErr};
use pigment::rpc;
use tracing::Instrument;

#[tokio::main]
async fn main() -> Result<()> {
    console_subscriber::init();

    let listen_addr = std::env::var("PIGMENT_ADDR").unwrap_or_else(|_| "0.0.0.0:4211".into());
    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .into_diagnostic()?;

    let server: &'static _ = Box::leak(Box::new(rpc::Server::new(Handler::new(
        sled::open(std::env::var("PIGMENT_DB").unwrap_or_else(|_| "db".into()))
            .into_diagnostic()
            .wrap_err("failed to open sled database")?,
    ))));

    tracing::info!("started listening on {}", listen_addr);
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
