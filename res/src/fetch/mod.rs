pub mod error;

use canvas::{
    client::{
        hyper::{client::connect::Connect, Method},
        pagination,
    },
    resource,
};
use fallible_stream::YieldError;
use futures::Stream;
use pin_project::pin_project;
use std::{pin::Pin, task::Poll};

pub use error::{Error, Result};

/// Responsible for fetching a resource from the underlying Canvas API.
pub trait Fetch<R>: Sized {
    type FetchAll: Stream<Item = Result<R>> + Send;
    fn fetch_all(self) -> Self::FetchAll;
}

impl<Conn> Fetch<resource::Course> for &canvas::Client<Conn>
where
    Conn: Clone + Connect + Send + Sync + Unpin + 'static,
{
    type FetchAll = YieldError<
        resource::Course,
        Error,
        CanvasItemStream<pagination::Items<'static, Conn, resource::Course>>,
    >;
    fn fetch_all(self) -> Self::FetchAll {
        YieldError::Ok(
            self.request(Method::GET, "/api/v1/courses")
                .paginate_owned(50)
                .map_err(Error::Canvas)? // TODO: adjust this per Canvas instance?
                .items::<resource::Course>()
                .into(),
        )
    }
}

#[pin_project]
pub struct CanvasItemStream<S>(#[pin] S);

impl<S> From<S> for CanvasItemStream<S> {
    fn from(s: S) -> Self {
        Self(s)
    }
}

impl<T, S: Stream<Item = canvas::client::Result<T>>> Stream for CanvasItemStream<S> {
    type Item = Result<T>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.project()
            .0
            .poll_next(cx)
            .map(|opt| opt.map(|res| res.map_err(Error::from)))
    }
}
