use futures::{stream::Empty, Stream};
use pin_project::pin_project;
use std::{
    convert::Infallible,
    ops::{ControlFlow, FromResidual, Try},
    pin::Pin,
    task::{self, Poll},
};

#[pin_project(project = YieldErrorProj)]
pub enum YieldError<T, E, S: Stream<Item = Result<T, E>>> {
    Ok(#[pin] S),
    Err(Option<E>),
}

impl<T, E, S: Stream<Item = Result<T, E>>> Stream for YieldError<T, E, S> {
    type Item = Result<T, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project() {
            YieldErrorProj::Ok(s) => s.poll_next(cx),
            YieldErrorProj::Err(e) => Poll::Ready(e.take().map(Err)),
        }
    }
}

impl<T, E, S: Stream<Item = Result<T, E>>> From<Result<S, E>> for YieldError<T, E, S> {
    #[inline]
    fn from(res: Result<S, E>) -> Self {
        match res {
            Ok(s) => Self::Ok(s),
            Err(e) => Self::Err(Some(e)),
        }
    }
}

impl<T, E, S: Stream<Item = Result<T, E>>> Try for YieldError<T, E, S> {
    type Output = S;
    type Residual = YieldError<Infallible, E, Empty<Result<Infallible, E>>>;

    fn from_output(output: Self::Output) -> Self {
        YieldError::Ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok(s) => ControlFlow::Continue(s),
            Self::Err(e) => ControlFlow::Break(YieldError::Err(e)),
        }
    }
}

impl<T, E, S: Stream<Item = Result<T, E>>> FromResidual for YieldError<T, E, S> {
    #[inline]
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            YieldError::Err(err) => Self::Err(err),
            _ => unreachable!(),
        }
    }
}

impl<T, E, S: Stream<Item = Result<T, E>>> FromResidual<Result<Infallible, E>>
    for YieldError<T, E, S>
{
    #[inline]
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Err(e) => YieldError::Err(Some(e)),
            _ => unreachable!(),
        }
    }
}
