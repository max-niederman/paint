use crate::Result;
use reqwest::{Method, RequestBuilder};
use serde::de::Deserialize;

pub use pagination::Pagination;

/// Canvas API Client
#[derive(Debug, Clone)]
pub struct Client {
    client: reqwest::Client,
    base_url: String,
}

impl Client {
    pub fn new<U: AsRef<str>>(base_url: U, auth: Option<Auth>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    if let Some(auth) = auth {
                        headers.insert(
                            reqwest::header::AUTHORIZATION,
                            match auth {
                                Auth::Bearer(token) => format!("Bearer {}", token)
                                    .try_into()
                                    .expect("auth token was invalid"),
                            },
                        );
                    }
                    headers
                })
                .build()
                .expect("while instantiating HTTP client"),
            base_url: base_url.as_ref().to_owned(),
        }
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    pub fn http_client(&self) -> &reqwest::Client {
        &self.client
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn request<P: AsRef<str>>(&self, method: Method, path: P) -> RequestBuilder {
        let url = format!("{}/{}", self.base_url, path.as_ref());
        self.client.request(method, url)
    }

    pub fn get<P: AsRef<str>>(&self, path: P) -> RequestBuilder {
        self.request(Method::GET, path)
    }

    pub fn post<P: AsRef<str>>(&self, path: P) -> RequestBuilder {
        self.request(Method::POST, path)
    }

    pub fn put<P: AsRef<str>>(&self, path: P) -> RequestBuilder {
        self.request(Method::PUT, path)
    }

    pub fn patch<P: AsRef<str>>(&self, path: P) -> RequestBuilder {
        self.request(Method::PATCH, path)
    }

    pub fn delete<P: AsRef<str>>(&self, path: P) -> RequestBuilder {
        self.request(Method::DELETE, path)
    }

    pub fn head<P: AsRef<str>>(&self, path: P) -> RequestBuilder {
        self.request(Method::HEAD, path)
    }
}

pub enum Auth {
    Bearer(String),
}

pub fn deserialize_from_slice<'a, T: Deserialize<'a>>(v: &'a [u8]) -> Result<T> {
    serde_json::from_slice(v)
        .map_err(|je| crate::Error::from_json_err(je, std::str::from_utf8(v).unwrap().to_string()))
}
pub struct ClientBuilder {
    base_url: String,
    auth: Option<Auth>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            base_url: "https://canvas.instructure.com".to_owned(),
            auth: None,
        }
    }
}

impl ClientBuilder {
    pub fn build(self) -> Client {
        Client::new(self.base_url, self.auth)
    }

    pub fn with_base_url<U: AsRef<str>>(mut self, base_url: U) -> Self {
        self.base_url = base_url.as_ref().to_owned();
        self
    }

    pub fn with_auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }
}

pub mod pagination {
    use std::collections::HashMap;

    use miette::{Diagnostic, IntoDiagnostic, Result};
    use reqwest::header::HeaderMap;
    use thiserror::Error;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Pagination<'h> {
        current: Option<&'h str>,
        next: Option<&'h str>,
        previous: Option<&'h str>,
        first: Option<&'h str>,
        last: Option<&'h str>,
    }

    macro_rules! impl_pagination_getter {
        ($name:ident) => {
            pub fn $name(&self) -> Result<&str> {
                self.$name
                    .ok_or_else(|| Error::MissingLink(stringify!($name)))
                    .into_diagnostic()
            }
        };
    }

    impl<'h> Pagination<'h> {
        pub fn from_headers(headers: &'h HeaderMap) -> Result<Self> {
            headers
                .get(reqwest::header::LINK)
                .ok_or(Error::MissingLinksHeader)
                .and_then(|links| {
                    links.to_str().map_err(|_| Error::MalformedLinkHeader {
                        src: std::str::from_utf8(links.as_bytes()).unwrap().to_string(),
                        message: "contained invalid characters",
                    })
                })
                .into_diagnostic()
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

        impl_pagination_getter!(current);
        impl_pagination_getter!(next);
        impl_pagination_getter!(previous);
        impl_pagination_getter!(first);
        impl_pagination_getter!(last);
    }

    #[derive(Debug, Error, Diagnostic)]
    pub enum Error {
        #[error("missing `Links` header")]
        MissingLinksHeader,

        #[error("missing pagination link with relevance `{0}`")]
        MissingLink(&'static str),

        #[error("failed to parse `Links` header: {message}")]
        MalformedLinkHeader {
            #[source_code]
            src: String,
            message: &'static str,
        },
    }

    #[cfg(test)]
    #[test]
    fn parse_from_header() {
        assert_eq!(
            Pagination::from_links_header(&"<https://canvas.instructure.com/api/v1/courses?page=2>; rel=\"current\", <https://canvas.instructure.com/api/v1/courses?page=3>; rel=\"next\", <https://canvas.instructure.com/api/v1/courses?page=1>; rel=\"prev\", <https://canvas.instructure.com/api/v1/courses?page=1>; rel=\"first\", <https://canvas.instructure.com/api/v1/courses?page=3>; rel=\"last\"").unwrap(),
            Pagination {
                current: Some("https://canvas.instructure.com/api/v1/courses?page=2"),
                next: Some("https://canvas.instructure.com/api/v1/courses?page=3"),
                previous: Some("https://canvas.instructure.com/api/v1/courses?page=1"),
                first: Some("https://canvas.instructure.com/api/v1/courses?page=1"),
                last: Some("https://canvas.instructure.com/api/v1/courses?page=3"),
            }
        )
    }
}
