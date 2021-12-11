pub mod error;
pub mod message;

pub use error::Error;
pub use message::{Request, Response};

use futures::{
    stream::{Stream, TryStreamExt},
    Sink, SinkExt,
};

impl Request {
    pub async fn send<T, E>(
        self,
        transport: &mut T,
    ) -> crate::Result<impl Stream<Item = Result<Result<Response, String>, Error>> + '_>
    where
        T: Stream<Item = Result<Result<Response, String>, E>> + Sink<Request, Error = E> + Unpin,
        E: 'static + std::error::Error + Send + Sync,
    {
        transport.send(self).await.map_err(Error::transport)?;

        Ok(transport.map_err(Error::transport))
    }
}

pub trait Handler {
    type Err: std::error::Error;
    type ResponseStream: Stream<Item = Result<Response, Self::Err>> + Unpin;
    fn handle(&self, request: Request) -> Self::ResponseStream;
}

impl<Err, Ret, Func> Handler for Func
where
    Err: std::error::Error,
    Ret: Stream<Item = Result<Response, Err>> + Unpin,
    Func: Fn(Request) -> Ret,
{
    type Err = Err;
    type ResponseStream = Ret;
    fn handle(&self, request: Request) -> Self::ResponseStream {
        self(request)
    }
}
