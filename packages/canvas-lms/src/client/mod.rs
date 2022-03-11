pub mod error;
pub mod pagination;
pub mod request;
pub mod response;

pub use error::{Error, Result};
pub use hyper;
pub use request::RequestBuilder;
pub use response::Response;

use hyper::{client::HttpConnector, Method};

// TODO: currently, we handle throttling only for paginated requests, but this causes problems when we make many paginated requests simultaneously
//       and doesn't even attempt to handle non-paginated responses, leading to errors in some cases.
//       at some point, we should move throttle handling to the `Client` so it acts on all requests, paginated or not.

#[derive(Debug, Clone)]
pub struct Client<Conn = HttpConnector> {
    hyper: hyper::Client<Conn>,

    auth: Option<Auth>,
    // TODO: store domain instead of URL prefix
    base_uri: String,
}

impl<Conn> Client<Conn> {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn request(&self, method: Method, path: impl Into<String>) -> RequestBuilder<'_, Conn> {
        RequestBuilder::new(self, method, path.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Auth {
    Bearer(String),
}

#[derive(Debug)]
pub struct ClientBuilder {
    auth: Option<Auth>,
    base_url: String,
}

impl ClientBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            auth: None,
            base_url: "https://canvas.instructure.com".to_string(),
        }
    }

    #[inline]
    pub fn build<Conn: Clone>(self, http: hyper::Client<Conn>) -> Client<Conn> {
        Client {
            hyper: http,
            auth: self.auth,
            base_uri: self.base_url,
        }
    }

    #[inline]
    #[must_use = "client builder methods create new builders"]
    pub fn auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }

    #[inline]
    #[must_use = "client builder methods create new builders"]
    pub fn base_url<U: Into<String>>(mut self, base_url: U) -> Self {
        self.base_url = base_url.into();
        self
    }
}

impl Default for ClientBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
