pub mod error;
pub mod pagination;
pub mod request;
pub mod response;

pub use error::{Error, Result};
pub use hyper;
pub use request::RequestBuilder;
pub use response::Response;

use hyper::{client::HttpConnector, Method};

#[derive(Debug, Clone)]
pub struct Client<Conn: Clone = HttpConnector> {
    hyper: hyper::Client<Conn>,

    auth: Option<Auth>,
    base_uri: String,
}

impl<Conn: Clone> Client<Conn> {
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
    pub fn auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }
    #[inline]
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