use crate::{
    message::{Request, Response},
    Error, Result,
};
use futures::{Stream, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};

pub struct Server<H: Handler> {
    handler: H,
}

impl<H: Handler> Server<H> {
    pub fn new(handler: H) -> Self {
        Self { handler }
    }

    pub async fn handle<T: Transport>(&self, transport: &mut T) -> Result<()> {
        let request = transport.read()?;
        let mut responses = self.handler.handle(request);

        while let Some(handled) = responses.next().await {
            let msg = match handled {
                Ok(res) => Ok(res),
                Err(err) => {
                    log::error!("handler failed:\n {}", err);
                    Err(err.to_string())
                },
            };
            transport.write(&msg)?;
        }
        Ok(())
    }
}

pub trait Handler {
    type Err: std::error::Error;
    type ResponseStream: Stream<Item = std::result::Result<Response, Self::Err>> + Unpin;
    fn handle(&self, request: Request) -> Self::ResponseStream;
}

impl<Err, Ret, Func> Handler for Func
where
    Err: std::error::Error,
    Ret: Stream<Item = std::result::Result<Response, Err>> + Unpin,
    Func: Fn(Request) -> Ret,
{
    type Err = Err;
    type ResponseStream = Ret;
    fn handle(&self, request: Request) -> Self::ResponseStream {
        self(request)
    }
}

pub trait Transport {
    fn read<M: DeserializeOwned>(&mut self) -> Result<M>;
    fn write<M: Serialize>(&mut self, msg: &M) -> Result<()>;
    fn close(self) -> Result<()>;
}

#[allow(drop_bounds)]
impl<T: Read + Write + Drop> Transport for T {
    fn read<M: DeserializeOwned>(&mut self) -> Result<M> {
        bincode::deserialize_from(self).map_err(Error::Transport)
    }
    fn write<M: Serialize>(&mut self, msg: &M) -> Result<()> {
        bincode::serialize_into(self, msg).map_err(Error::Transport)
    }
    fn close(self) -> Result<()> {
        Ok(drop(self))
    }
}
