extern crate canvas_lms as canvas;

mod fetch;

use async_bincode::AsyncBincodeStream;
use futures::{Sink, SinkExt, Stream, StreamExt};
use miette::{IntoDiagnostic, WrapErr, Result};
use pigment::rpc;

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

            let mut transport = AsyncBincodeStream::from(socket).for_async();

            match handle(&mut transport).await {
                Ok(()) => log::debug!("finished handling connection from {}", remote),
                Err(e) => log::error!("fatal error handling connection from {}: {}", remote, e),
            }
        });
    }
}

async fn handle<T, E>(transport: &mut T) -> Result<()>
where
    T: Stream<Item = Result<rpc::Request, E>>
        + Sink<Result<rpc::Response, String>, Error = E>
        + Unpin,
    E: std::error::Error + Send + Sync + 'static,
{
    transport
        .send(Ok(rpc::Response::UpdateResult {
            downloaded: 0,
            updated: 0,
            canvas_cost: 0.0,
            canvas_time: 0.0,
        }))
        .await
        .into_diagnostic()
        .wrap_err("while sending test response")?;

    Ok(())
}
