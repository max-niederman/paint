use super::{Client, Error, Response, Result};
use futures::{ready, Future, FutureExt, Stream, StreamExt};
use futures_timer::Delay;
use hyper::{
    client::connect::Connect,
    header::{self, HeaderMap},
    Body, Method, StatusCode, Uri,
};
use serde::de::DeserializeOwned;
use std::{
    borrow::Cow,
    collections::HashMap,
    convert::Infallible,
    mem::MaybeUninit,
    ops::FromResidual,
    pin::Pin,
    task::{self, Poll},
    time::Duration,
};

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Pagination<'c, Conn: Clone> {
    client: Cow<'c, Client<Conn>>,
    headers: HeaderMap,
    state: PaginationState,
}

#[derive(Debug)]
enum PaginationState {
    Throttled {
        timer: Delay,
        uri: Uri,
    },
    AwaitingResponse {
        resp_fut: hyper::client::ResponseFuture,
        uri: Uri,
    },
    Finished,
}

impl<'c, Conn> Pagination<'c, Conn>
where
    Conn: Connect + Clone + Send + Sync + 'static,
{
    #[inline]
    pub(super) fn new(client: Cow<'c, Client<Conn>>, headers: HeaderMap, uri: Uri) -> Result<Self> {
        Ok(Self {
            state: PaginationState::awaiting_response(&client, uri, headers.clone())?,
            client,
            headers,
        })
    }

    #[inline]
    pub fn items<T: DeserializeOwned>(self) -> Items<'c, Conn, T> {
        Items {
            pagination: self,
            items: Vec::new(),
            state: ItemsState::AwaitingPage,
        }
    }
}

struct PaginationStateTransition<T> {
    new: PaginationState,
    ret: T,
}

impl<T> FromResidual<Result<Infallible>> for PaginationStateTransition<T>
where
    T: FromResidual<Result<Infallible>>,
{
    fn from_residual(residual: Result<Infallible>) -> Self {
        Self {
            new: PaginationState::Finished,
            ret: T::from_residual(residual),
        }
    }
}

impl PaginationState {
    #[inline(always)]
    fn transition<Ret, F: FnOnce(Self) -> PaginationStateTransition<Ret>>(&mut self, f: F) -> Ret {
        unsafe {
            //   EXPL: first, we reinterpret `self` as a `&mut MaybeUninit<Self>` and bind it to `this`
            //         this is completely safe on its own, so I'm not really sure why there's no safe function for it in `std`
            let this = std::mem::transmute::<&mut Self, &mut MaybeUninit<Self>>(self);

            //   EXPL: then, we _bitwise_ copy the [`Self`] out of `this`, call `f` with it and immediately overwrite `this`
            //         with the new state returned. this has the effect of moving out of and then into `this`
            // SAFETY: assuming `this` to be initialized inbetween the call to [`MaybeUninit::assume_init_read`] and the
            //         call to [`MaybeUninit::write`] is unsafe
            let PaginationStateTransition { new, ret } = f(this.assume_init_read());
            this.write(new);

            ret
        }
    }

    #[inline(always)]
    fn awaiting_response<Conn>(client: &Client<Conn>, uri: Uri, headers: HeaderMap) -> Result<Self>
    where
        Conn: Connect + Clone + Send + Sync + 'static,
    {
        let mut builder = hyper::Request::builder()
            .method(Method::GET)
            .uri(uri.clone());

        // `builder` will not error because [`<Uri as TryFrom<Uri>>::try_from`] is infallible,
        // so unwrapping the result will never panic
        *builder.headers_mut().unwrap() = headers;

        Ok(Self::AwaitingResponse {
            resp_fut: client.hyper.request(builder.body(Body::empty())?),
            uri,
        })
    }
}

// TODO: should we write a version of [`Pagination`] which requests all pages
//       simultaneously, destroying the order but improving performance if ordering
//       is unnecessary? this may not be useful because we don't want to stress the
//       underlying Canvas instances more than necessary
// ----: the Canvas API documentation dicatates that pagination links should be treated as
//       opaque, so we may not want to do this regardless

impl<'c, Conn> Stream for Pagination<'c, Conn>
where
    Conn: Connect + Clone + Send + Sync + Unpin + 'static,
{
    type Item = Result<Response>;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let client = self.client.clone();
        let req_headers = self.headers.clone();
        self.state.transition(|mut state| match state {
            PaginationState::AwaitingResponse {
                ref mut resp_fut,
                ref uri,
            } => match resp_fut.poll_unpin(cx) {
                Poll::Ready(Ok(response)) => tracing::debug_span!("handling pagination response", 
                %uri,
                cost = response.headers().get("X-Request-Cost").and_then(|hv| Some(hv.to_str().ok()?.parse::<f32>().ok())),
                ratelimit_remaining = response.headers().get("X-Rate-Limit-Remaining").and_then(|hv| Some(hv.to_str().ok()?.parse::<f32>().ok()))
            ).in_scope(|| match response.status() {
                    StatusCode::OK => {
                        tracing::trace!("recieved page");

                        PaginationStateTransition {
                            new: match PaginationLinks::from_headers(&response.headers())?.next() {
                                Ok(next) => PaginationState::awaiting_response(
                                    &client,
                                    next.clone(),
                                    req_headers,
                                )?,
                                Err(_) => PaginationState::Finished,
                            },
                            ret: Poll::Ready(Some(Ok(response.into()))),
                        }
                    }

                    StatusCode::FORBIDDEN => {
                        tracing::warn!("request throttled");

                        PaginationStateTransition {
                            new: PaginationState::Throttled {
                                timer: Delay::new(Duration::from_secs_f32(2.0)), // TODO: adjust by ratelimit remaining and per Canvas instance
                                uri: uri.clone(),
                            },
                            ret: {
                                cx.waker().wake_by_ref();
                                Poll::Pending
                            },
                        }
                    }

                    StatusCode::UNAUTHORIZED => {
                        tracing::error!(message = "incorrect authorization", auth_header = ?response.headers().get(header::AUTHORIZATION));

                        PaginationStateTransition {
                            new: PaginationState::Finished,
                            ret: Poll::Ready(Some(Err(Error::Unauthorized)))
                        }
                    }

                    code => {
                        tracing::warn!(message = "recieved response with unknown status code", %code);

                        PaginationStateTransition {
                            new: PaginationState::Finished,
                            ret: Poll::Ready(Some(Err(Error::UnknownHttpStatus {
                                code,
                                headers: response.headers().clone(),
                                response: response.into(),
                            })))
                        }
                    },
                }),
                Poll::Ready(Err(err)) => PaginationStateTransition::from_residual(Err(err.into())),
                Poll::Pending => PaginationStateTransition {
                    new: state,
                    ret: Poll::Pending,
                },
            },
            PaginationState::Throttled {
                ref mut timer,
                ref uri,
            } => match timer.poll_unpin(cx) {
                Poll::Ready(()) => PaginationStateTransition {
                    new: PaginationState::awaiting_response(&client, uri.clone(), req_headers)?,
                    ret: {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    },
                },
                Poll::Pending => PaginationStateTransition {
                    new: state,
                    ret: Poll::Pending,
                },
            },
            PaginationState::Finished => PaginationStateTransition {
                new: PaginationState::Finished,
                ret: Poll::Ready(None),
            },
        })
    }
}

#[must_use = "streams do nothing unless polled"]
pub struct Items<'c, Conn: Clone, T: DeserializeOwned> {
    pagination: Pagination<'c, Conn>,
    items: Vec<T>, // this list is reversed so we don't have to pop items from the front
    state: ItemsState<T>,
}

enum ItemsState<T> {
    Deserializing(Pin<Box<dyn Future<Output = Result<Vec<T>>> + Send>>),
    AwaitingPage,
}

impl<'c, Conn: Clone, T> Stream for Items<'c, Conn, T>
where
    T: DeserializeOwned + Unpin + 'static,
    Conn: Connect + Clone + Send + Sync + Unpin + 'static,
{
    type Item = Result<T>;

    #[inline]
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(item) = self.items.pop() {
            // if we have an item already deserialized, yield it immediately
            cx.waker().wake_by_ref();
            Poll::Ready(Some(Ok(item)))
        } else {
            match self.state {
                ItemsState::Deserializing(ref mut deser_fut) => {
                    self.items = ready!(deser_fut.poll_unpin(cx))?;
                    self.items.reverse();

                    // transition to waiting for next page
                    self.state = ItemsState::AwaitingPage;

                    cx.waker().wake_by_ref();
                    Poll::Ready(self.items.pop().map(Ok))
                }
                ItemsState::AwaitingPage => {
                    let page = ready!(self.pagination.poll_next_unpin(cx));

                    match page {
                        Some(Ok(response)) => {
                            self.state = ItemsState::Deserializing(Box::pin(
                                response.deserialize::<Vec<T>>(),
                            ));
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        }
                        Some(Err(e)) => Poll::Ready(Some(Err(e))),
                        None => Poll::Ready(None),
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationLinks {
    current: Option<Uri>,
    next: Option<Uri>,
    previous: Option<Uri>,
    first: Option<Uri>,
    last: Option<Uri>,
}

macro_rules! pagination_links_getter {
    ($name:ident) => {
        #[inline]
        pub fn $name(&self) -> Result<&Uri> {
            self.$name
                .as_ref()
                .ok_or_else(|| Error::MissingPaginationLink(stringify!($name)).into())
        }
    };
}

impl PaginationLinks {
    #[inline]
    pub fn from_headers(headers: &HeaderMap) -> Result<Self> {
        headers
            .get(header::LINK)
            .ok_or(Error::MissingLinksHeader)
            .and_then(|links| {
                links.to_str().map_err(|_| Error::MalformedLinkHeader {
                    src: std::str::from_utf8(links.as_bytes()).unwrap().to_string(),
                    message: "contained invalid characters",
                })
            })
            .map_err(Into::into)
            .and_then(Self::from_links_header)
    }

    /// Parse pagination links from a response header as per W3C 9707.
    #[inline]
    pub fn from_links_header<H: AsRef<str> + ?Sized>(header: &H) -> Result<Self> {
        let mut links = HashMap::<&str, Uri>::with_capacity(5);
        for link in header.as_ref().split(',') {
            let (url, rel) =
                link.trim()
                    .split_once(';')
                    .ok_or_else(|| Error::MalformedLinkHeader {
                        src: link.to_string(),
                        message: "missing semicolon",
                    })?;

            let url = &url[1..url.len() - 1];

            let rel = rel
                .trim_start()
                .split_once('=')
                .ok_or_else(|| Error::MalformedLinkHeader {
                    src: rel.trim_start().to_string(),
                    message: "missing relevance",
                })?
                .1;
            let rel = &rel[1..rel.len() - 1];

            links.insert(rel, url.try_into().map_err(hyper::http::Error::from)?);
        }

        Ok(Self {
            current: links.remove("current"),
            next: links.remove("next"),
            previous: links.remove("prev"),
            first: links.remove("first"),
            last: links.remove("last"),
        })
    }

    pagination_links_getter!(current);
    pagination_links_getter!(next);
    pagination_links_getter!(previous);
    pagination_links_getter!(first);
    pagination_links_getter!(last);
}

#[cfg(test)]
#[test]
fn parse_from_header() {
    assert_eq!(
        PaginationLinks::from_links_header(&"<https://canvas.instructure.com/api/v1/courses?page=2>; rel=\"current\", <https://canvas.instructure.com/api/v1/courses?page=3>; rel=\"next\", <https://canvas.instructure.com/api/v1/courses?page=1>; rel=\"prev\", <https://canvas.instructure.com/api/v1/courses?page=1>; rel=\"first\", <https://canvas.instructure.com/api/v1/courses?page=3>; rel=\"last\"").unwrap(),
        PaginationLinks {
            current: Some("https://canvas.instructure.com/api/v1/courses?page=2".try_into().unwrap()),
            next: Some("https://canvas.instructure.com/api/v1/courses?page=3".try_into().unwrap()),
            previous: Some("https://canvas.instructure.com/api/v1/courses?page=1".try_into().unwrap()),
            first: Some("https://canvas.instructure.com/api/v1/courses?page=1".try_into().unwrap()),
            last: Some("https://canvas.instructure.com/api/v1/courses?page=3".try_into().unwrap()),
        }
    )
}
