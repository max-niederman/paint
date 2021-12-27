pub mod error;
pub mod message;

use std::fmt::Display;
use futures::{io, prelude::*};

pub use error::{Error, Result};
pub use message::{Request, Response};

#[derive(Debug, Clone, Copy)]
pub struct Server<H> {
    handler: H,
}

impl<'h, H: Handler<'h>> Server<H> {
    pub fn new(handler: H) -> Self {
        Self { handler }
    }

    pub async fn handle<T, E>(&'h self, transport: &mut T) -> Result<()>
    where
        T: Stream<Item = Result<Request, E>> + Sink<Result<Response, String>, Error = E> + Unpin,
        E: 'static + std::error::Error + Send + Sync,
    {
        let request = transport
            .next()
            .await
            .ok_or_else(|| {
                Error::Transport(Box::new(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unexpected EOF while waiting for request",
                )))
            })?
            .map_err(Error::transport)?;

        self.handler
            .handle(request)
            .map(|res| {
                Ok(res.map_err(|error| {
                    tracing::error!(message = "handler failed", %error);
                    error.to_string()
                }))
            })
            .forward(transport)
            .await
            .map_err(Error::transport)?;

        Ok(())
    }
}

impl Request {
    pub async fn send<T, E>(
        self,
        transport: &mut T,
    ) -> Result<impl Stream<Item = Result<Result<Response, String>>> + '_>
    where
        T: Stream<Item = Result<Result<Response, String>, E>> + Sink<Request, Error = E> + Unpin,
        E: 'static + std::error::Error + Send + Sync,
    {
        transport.send(self).await.map_err(Error::transport)?;

        Ok(transport.map_err(Error::transport))
    }
}

pub trait Handler<'h> {
    type Err: Display;
    type ResponseStream: Stream<Item = Result<Response, Self::Err>> + Unpin + 'h;
    fn handle(&'h self, request: Request) -> Self::ResponseStream;
}

impl<'h, Err, Ret, Func> Handler<'h> for Func
where
    Err: Display,
    Ret: Stream<Item = Result<Response, Err>> + Unpin + 'h,
    Func: Fn(Request) -> Ret,
{
    type Err = Err;
    type ResponseStream = Ret;
    fn handle(&'h self, request: Request) -> Self::ResponseStream {
        self(request)
    }
}
