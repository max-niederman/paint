use async_stream::stream;
use canvas::{
    client::hyper::{client::connect::Connect, Method},
    resource::{self, Resource},
};
use futures::{Stream, StreamExt};
use miette::{IntoDiagnostic, Result, WrapErr};
use std::pin::Pin;

/// Responsible for fetching a resource from the underlying Canvas API.
pub trait Fetch<'c>: Sized {
    type FetchAll: Stream<Item = Result<Self>> + Send + 'c;
    fn fetch_all<Conn>(client: &'c canvas::Client<Conn>) -> Result<Self::FetchAll>
    where
        Conn: Connect + Clone + Send + Sync + 'static;
}

impl<'c> Fetch<'c> for resource::Course {
    type FetchAll = Pin<Box<dyn Stream<Item = Result<Self>> + Send + 'c>>;
    fn fetch_all<Conn>(client: &'c canvas::Client<Conn>) -> Result<Self::FetchAll>
    where
        Conn: Connect + Clone + Send + Sync + 'static,
    {
        Ok(Box::pin(
            client
                .request(Method::GET, "/api/v1/courses")
                .paginate(1)?
                .items::<Self>()
                .map(IntoDiagnostic::into_diagnostic),
        ))
    }
}
