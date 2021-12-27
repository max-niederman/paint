pub mod error;
pub mod tiered;

use canvas::{
    client::{
        hyper::{client::connect::Connect, Method},
        pagination,
    },
    resource,
};
use fallible_stream::YieldError;
use futures::prelude::*;
use pin_project::pin_project;
use std::{pin::Pin, task::Poll};

pub use error::{Error, Result};

/// Responsible for fetching a resource from the underlying Canvas API.
pub trait Fetch<R>: Sized {
    type Dependency;

    type FetchStream: Stream<Item = Result<R>>;
    fn fetch(&self, dependency: &Self::Dependency) -> Self::FetchStream;

    fn fetch_independent(&self) -> Self::FetchStream
    where
        Self::Dependency: Default,
    {
        self.fetch(&Default::default())
    }
}

impl<Conn> Fetch<resource::Course> for canvas::Client<Conn>
where
    Conn: Clone + Connect + Send + Sync + Unpin + 'static,
{
    type Dependency = ();

    type FetchStream =
        YieldError<CanvasItemStream<pagination::Items<'static, Conn, resource::Course>>>;
    fn fetch(&self, _: &Self::Dependency) -> Self::FetchStream {
        YieldError::Ok(
            self.request(Method::GET, "/api/v1/courses")
                .paginate_owned(50)
                .map_err(Error::Canvas)? // TODO: adjust this per Canvas instance?
                .items::<resource::Course>()
                .into(),
        )
    }
}

impl<Conn> Fetch<resource::Assignment> for canvas::Client<Conn>
where
    Conn: Clone + Connect + Send + Sync + Unpin + 'static,
{
    type Dependency = resource::Course;

    type FetchStream =
        YieldError<CanvasItemStream<pagination::Items<'static, Conn, resource::Assignment>>>;
    fn fetch(&self, dependency: &Self::Dependency) -> Self::FetchStream {
        YieldError::Ok(
            self.request(
                Method::GET,
                format!("/api/v1/courses/{}/assignments", dependency.id),
            )
            .paginate_owned(50)
            .map_err(Error::Canvas)?
            .items::<resource::Assignment>()
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
