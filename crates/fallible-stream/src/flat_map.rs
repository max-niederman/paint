use futures::{prelude::*, ready};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{self, Poll},
};

#[pin_project]
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct TryFlatMap<S, U, F>
where
    S: TryStream,
    U: TryStream<Error = S::Error>,
{
    #[pin]
    source: S,
    #[pin]
    map: F,

    #[pin]
    current: Option<U>,
}

impl<S, U, F> TryFlatMap<S, U, F>
where
    S: TryStream,
    U: TryStream<Error = S::Error>,
{
    #[inline]
    pub fn new(source: S, map: F) -> Self {
        Self {
            source,
            map,
            current: None,
        }
    }
}

impl<S, U, F> Stream for TryFlatMap<S, U, F>
where
    S: TryStream,
    U: TryStream<Error = S::Error>,
    F: FnMut(S::Ok) -> U + Unpin,
{
    type Item = Result<U::Ok, U::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(current) = self.as_mut().project().current.as_pin_mut() {
            match ready!(current.try_poll_next(cx)) {
                Some(item) => Poll::Ready(Some(item)),
                None => {
                    self.as_mut().project().current.set(None);
                    self.poll_next(cx)
                }
            }
        } else {
            let item = ready!(self.as_mut().project().source.try_poll_next(cx));
            match item {
                Some(Ok(item)) => {
                    let mut this = self.as_mut().project();
                    this.current.set(Some((this.map)(item)));

                    self.poll_next(cx)
                }
                Some(Err(err)) => Poll::Ready(Some(Err(err))),
                None => Poll::Ready(None),
            }
        }
    }
}

#[pin_project]
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct TryFlatMapSelect<S, U, F>
where
    S: TryStream,
    U: TryStream<Error = S::Error> + Unpin,
{
    // NOTE: we don't just use a [`Fuse<S>`] here because it doesn't play well with [`TryStream`]
    #[pin]
    source: S,
    source_finished: bool,

    #[pin]
    map: F,

    #[pin]
    streams: Vec<U>,
}

impl<S, U, F> TryFlatMapSelect<S, U, F>
where
    S: TryStream,
    U: TryStream<Error = S::Error> + Unpin,
{
    #[inline]
    pub fn new(source: S, map: F) -> Self
    where
        F: FnMut(S::Ok) -> U + Unpin,
    {
        Self {
            map,
            streams: Vec::with_capacity(source.size_hint().0),
            source,
            source_finished: false,
        }
    }
}

impl<S, U, F> Stream for TryFlatMapSelect<S, U, F>
where
    S: TryStream,
    U: TryStream<Error = S::Error> + Unpin,
    F: FnMut(S::Ok) -> U + Unpin,
{
    type Item = Result<U::Ok, U::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        if !*this.source_finished {
            match this.source.try_poll_next(cx) {
                Poll::Ready(Some(Ok(item))) => this.streams.push((this.map)(item)),
                Poll::Ready(Some(Err(item))) => {
                    cx.waker().wake_by_ref();
                    return Poll::Ready(Some(Err(item)));
                }
                Poll::Ready(None) => *this.source_finished = true,
                _ => {}
            }
        }

        let mut i = 0;
        while i < this.streams.len() {
            match this.streams[i].try_poll_next_unpin(cx) {
                Poll::Ready(Some(item)) => {
                    cx.waker().wake_by_ref();
                    return Poll::Ready(Some(item));
                }
                Poll::Ready(None) => {
                    // we don't increment the index here becuase we just swapped a new stream into the current index
                    this.streams.swap_remove(i);
                }
                Poll::Pending => {
                    i += 1;
                }
            }
        }

        if *this.source_finished && this.streams.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}
