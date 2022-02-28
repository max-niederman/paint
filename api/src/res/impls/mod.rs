use super::*;
use canvas_lms::{
    client::{
        hyper::{client::connect::Connect, Method},
        pagination::Items,
        Client,
    },
};
use fallible_stream::YieldError;
use futures::{
    prelude::*,
    task::{self, Poll},
};
use std::pin::Pin;
use poem::{error::ResponseError, Result};

mod course;
mod view;

mod prelude {
    pub use super::super::{Node, HomoNode, FetchAll, FetchOne};
}

pub struct CanvasError(canvas_lms::client::Error);
impl ResponseError for CanvasError {
    fn status(&self) -> reqwest::StatusCode {
        use canvas_lms::client::Error;
        match self.0 {
            Error::HitRatelimit { .. } => reqwest::StatusCode::TOO_MANY_REQUESTS,
            _ => reqwest::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn as_response(&self) -> poem::Response
    where
        Self: std::error::Error + Send + Sync + 'static,
    {
        poem::Response::builder()
            .status(self.status())
            .body(self.0.to_string())
    }
}

pub struct CanvasItemStream<S>(pub S);
impl<S: TryStream<Error = canvas_lms::client::Error>> Stream for CanvasItemStream<S> {
    type Item = Result<S::Ok, CanvasError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0) };
        stream
            .try_poll_next(cx)
            .map(|item| item.map(|res| res.map_err(CanvasError)))
    }
}
impl<S> From<S> for CanvasItemStream<S> {
    fn from(s: S) -> Self {
        CanvasItemStream(s)
    }
}