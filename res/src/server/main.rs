#![feature(box_patterns)]
#![feature(async_closure)]
#![feature(once_cell)]

extern crate canvas_lms as canvas;

mod fetch;
mod handler;

use async_bincode::AsyncBincodeStream;
use futures::{future, StreamExt};
use handler::Handler;
use miette::{IntoDiagnostic, Result, WrapErr};
use pigment::rpc;
use std::lazy::SyncOnceCell;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let listen_addr = std::env::var("PIGMENT_ADDR").unwrap_or_else(|_| "0.0.0.0:4211".into());
    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .into_diagnostic()?;

    static SERVER: SyncOnceCell<rpc::Server<Handler>> = SyncOnceCell::new();
    SERVER
        .set(rpc::Server::new(Handler::new(
            sled::open(std::env::var("PIGMENT_DB").unwrap_or_else(|_| "db".into()))
                .into_diagnostic()
                .wrap_err("failed to open sled database")?,
        )))
        .expect("SERVER is already initialized");

    log::info!("listening on {}", listen_addr);
    loop {
        let (socket, remote) = listener.accept().await.into_diagnostic()?;
        tokio::spawn(async move {
            log::debug!("accepted connection from {}", remote);

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

            match SERVER.get().unwrap().handle(&mut transport).await {
                Ok(()) => log::debug!("finished handling connection from {}", remote),
                Err(e) => log::error!("fatal error handling connection from {}: {}", remote, e),
            }
        });
    }
}
