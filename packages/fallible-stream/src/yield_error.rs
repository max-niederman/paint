use futures::{prelude::*, stream::Empty};
use pin_project::pin_project;
use std::{
    convert::Infallible,
    hint::unreachable_unchecked,
    ops::{ControlFlow, FromResidual, Try},
    pin::Pin,
    task::{self, Poll},
};

#[pin_project(project = YieldErrorProj)]
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub enum YieldError<S: TryStream> {
    Ok(#[pin] S),
    Err(Option<S::Error>),
}

impl<S: TryStream> Stream for YieldError<S> {
    type Item = Result<S::Ok, S::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project() {
            YieldErrorProj::Ok(s) => s.try_poll_next(cx),
            YieldErrorProj::Err(e) => Poll::Ready(e.take().map(Err)),
        }
    }
}

impl<S: TryStream> From<Result<S, S::Error>> for YieldError<S> {
    #[inline]
    fn from(res: Result<S, S::Error>) -> Self {
        match res {
            Ok(s) => Self::Ok(s),
            Err(e) => Self::Err(Some(e)),
        }
    }
}

impl<S: TryStream> Try for YieldError<S> {
    type Output = S;
    type Residual = YieldError<Empty<Result<Infallible, S::Error>>>;

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

impl<S: TryStream> FromResidual for YieldError<S> {
    #[inline]
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            YieldError::Err(err) => Self::Err(err),
            _ => unreachable!(),
        }
    }
}

impl<S: TryStream> FromResidual<Result<Infallible, S::Error>> for YieldError<S> {
    #[inline]
    fn from_residual(residual: Result<Infallible, S::Error>) -> Self {
        match residual {
            Err(e) => YieldError::Err(Some(e)),
            _ => unsafe { unreachable_unchecked() },
        }
    }
}
