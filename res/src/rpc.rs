use crate::message::{Request, Response};
use futures::{
    io,
    stream::{Stream, StreamExt},
    Sink, SinkExt,
};

pub struct Server<H: Handler> {
    handler: H,
}

impl<H: Handler> Server<H> {
    pub fn new(handler: H) -> Self {
        Self { handler }
    }

    pub async fn handle<T>(&self, transport: &mut T) -> crate::Result<()>
    where
        T: Stream<Item = Request> + Sink<Result<Response, String>> + Unpin,
        <T as Sink<Result<Response, String>>>::Error: std::error::Error + 'static,
    {
        let request = transport.next().await.ok_or_else(|| {
            crate::Error::Transport(Box::new(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "unexpected EOF while waiting for request",
            )))
        })?;

        self.handler
            .handle(request)
            .map(|res| Ok(res.map_err(|e| e.to_string())))
            .forward(transport)
            .await
            .map_err(|e| crate::Error::Transport(Box::new(e)))?;

        Ok(())
    }
}

impl Request {
    pub async fn send<T>(
        self,
        transport: &mut T,
    ) -> crate::Result<impl Stream<Item = Result<Response, String>> + '_>
    where
        T: Stream<Item = Result<Response, String>> + Sink<Request> + Unpin,
        <T as Sink<Request>>::Error: std::error::Error + 'static,
    {
        transport
            .send(self)
            .await
            .map_err(|source| crate::Error::Transport(Box::new(source)))?;
        Ok(transport)
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
