use super::{Client, Error, Response, Result};
use futures::{ready, Future, FutureExt, Stream, StreamExt};
use hyper::{
    client::connect::Connect,
    header::{self, HeaderMap},
    Body, Method, Uri,
};
use serde::de::DeserializeOwned;
use std::{collections::HashMap, pin::Pin, task::Poll};

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Pagination<'c, Conn: Clone> {
    client: &'c Client<Conn>,
    headers: HeaderMap,

    // current future request to be polled; if None, then the pagination is finished
    current_req: Option<hyper::client::ResponseFuture>,
}

impl<'c, Conn> Pagination<'c, Conn>
where
    Conn: Connect + Clone + Send + Sync + 'static,
{
    pub(super) fn new(client: &'c Client<Conn>, headers: HeaderMap, uri: Uri) -> Result<Self> {
        Ok(Pagination {
            client,
            headers: headers.clone(),
            current_req: Some(client.http.request({
                let mut builder = hyper::Request::builder().method(Method::GET).uri(uri);
                *builder
                    .headers_mut()
                    .expect("pagination request builder must not error") = headers;
                builder.body(Body::empty())?
            })),
        })
    }

    pub fn items<T: DeserializeOwned>(self) -> Items<'c, Conn, T> {
        Items {
            pagination: self,
            items: Vec::new(),
            state: ItemsState::WaitingForNextPage,
        }
    }
}

impl<'c, Conn> Stream for Pagination<'c, Conn>
where
    Conn: Connect + Clone + Send + Sync + 'static,
{
    type Item = Result<Response>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let current_req = match self.current_req.as_mut() {
            Some(fut) => fut,
            None => return Poll::Ready(None),
        };

        let res = ready!(current_req.poll_unpin(cx))
            .map(Response::from)
            .map_err(Error::from);

        match res {
            Ok(http_resp) => {
                let response: Response = http_resp.into();
                let links = response.pagination_links()?;

                self.current_req = match links.next {
                    Some(next) => Some(self.client.http.request({
                        log::trace!("requesting next page: {}", next);

                        let mut builder = hyper::Request::builder().method(Method::GET).uri(next);
                        *builder
                            .headers_mut()
                            .expect("pagination request builder must not error") =
                            self.headers.clone();
                        builder.body(Body::empty())?
                    })),
                    None => None,
                };

                Poll::Ready(Some(Ok(response)))
            }
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}

#[must_use = "streams do nothing unless polled"]
pub struct Items<'c, Conn: Clone, T: DeserializeOwned> {
    pagination: Pagination<'c, Conn>,
    items: Vec<T>, // NOTE: this list is reversed so we don't have to pop items from the front
    state: ItemsState<T>,
}

enum ItemsState<T> {
    Deserializing(Pin<Box<dyn Future<Output = Result<Vec<T>>> + Send>>),
    WaitingForNextPage,
}

impl<'c, Conn: Clone, T> Stream for Items<'c, Conn, T>
where
    T: DeserializeOwned + Unpin + 'static,
    Conn: Connect + Clone + Send + Sync + 'static,
{
    type Item = Result<T>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(item) = self.items.pop() {
            // if we have an item already deserialized, yield it immediately
            log::trace!("yielding item immediately");

            cx.waker().wake_by_ref();
            Poll::Ready(Some(Ok(item)))
        } else {
            match &mut self.state {
                ItemsState::Deserializing(deser_fut) => {
                    self.items = ready!(deser_fut.poll_unpin(cx))?;
                    self.items.reverse();

                    // transition to waiting for next page
                    self.state = ItemsState::WaitingForNextPage;

                    cx.waker().wake_by_ref();
                    Poll::Ready(self.items.pop().map(Ok))
                }
                ItemsState::WaitingForNextPage => {
                    let page = ready!(self.pagination.poll_next_unpin(cx));

                    match page {
                        Some(Ok(response)) => {
                            self.state = ItemsState::Deserializing(Box::pin(
                                response.deserialize::<Vec<T>>(),
                            ));
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        },
                        Some(Err(e)) => Poll::Ready(Some(Err(e))),
                        None => Poll::Ready(None),
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaginationLinks<'h> {
    current: Option<&'h str>,
    next: Option<&'h str>,
    previous: Option<&'h str>,
    first: Option<&'h str>,
    last: Option<&'h str>,
}

macro_rules! pagination_links_getter {
    ($name:ident) => {
        pub fn $name(&self) -> Result<&str> {
            self.$name
                .ok_or_else(|| Error::MissingPaginationLink(stringify!($name)).into())
        }
    };
}

impl<'h> PaginationLinks<'h> {
    pub fn from_headers(headers: &'h HeaderMap) -> Result<Self> {
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
    pub fn from_links_header<H: AsRef<str> + ?Sized>(header: &'h H) -> Result<Self> {
        let mut links = HashMap::<&str, &str>::with_capacity(5);
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

            links.insert(rel, url);
        }

        Ok(Self {
            current: links.get("current").copied(),
            next: links.get("next").copied(),
            previous: links.get("prev").copied(),
            first: links.get("first").copied(),
            last: links.get("last").copied(),
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
            current: Some("https://canvas.instructure.com/api/v1/courses?page=2"),
            next: Some("https://canvas.instructure.com/api/v1/courses?page=3"),
            previous: Some("https://canvas.instructure.com/api/v1/courses?page=1"),
            first: Some("https://canvas.instructure.com/api/v1/courses?page=1"),
            last: Some("https://canvas.instructure.com/api/v1/courses?page=3"),
        }
    )
}
