use futures::{prelude::*, ready, stream::Fuse};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{self, Poll},
};

#[pin_project]
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
                Some(item) => return Poll::Ready(Some(item)),
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
