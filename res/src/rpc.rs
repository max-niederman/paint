use crate::{
    message::{Request, Response, ResponseBody},
    Error, Result,
};
use futures::Stream;
use miette::ReportHandler;
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};

pub struct Server<T: Transport, H: Handler> {
    transport: T,
    handler: H,
}

impl<T: Transport, H: Handler> Server<T, H> {
    pub fn new(transport: T, handler: H) -> Self {
        Self { transport, handler }
    }

    pub async fn serve(&mut self) -> Result<!> {
        let mut i = 0;
        loop {
            let request = self.transport.read()?;
            unimplemented!()
        }
    }
}

pub trait Handler {
    type Err: std::error::Error;
    type ResponseStream: Stream<Item = std::result::Result<ResponseBody, Self::Err>>;
    fn handle(&self, request: Request) -> Self::ResponseStream;
}

impl<Err, Ret, Func> Handler for Func
where
    Err: std::error::Error,
    Ret: Stream<Item = std::result::Result<ResponseBody, Err>>,
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
}

impl<T: Read + Write> Transport for T {
    fn read<M: DeserializeOwned>(&mut self) -> Result<M> {
        bincode::deserialize_from(self)
    }
    fn write<M: Serialize>(&mut self, msg: &M) -> Result<()> {
        bincode::serialize_into(self, msg)
    }
}
