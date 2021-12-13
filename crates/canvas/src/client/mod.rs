pub mod error;
pub mod request;
pub mod response;
pub mod pagination;

pub use error::{Error, Result};
pub use request::RequestBuilder;
pub use response::Response;

use hyper::Method;

#[derive(Debug, Clone)]
pub struct Client<Conn: Clone> {
    hyper: hyper::Client<Conn>,

    auth: Option<Auth>,
    base_uri: String,
}

impl<Conn: Clone> Client<Conn> {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn request<'p, P: ToString>(&self, method: Method, path: P) -> RequestBuilder<'_, Conn> {
        RequestBuilder::new(self, method, path.to_string())
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
    pub fn new() -> Self {
        Self {
            auth: None,
            base_url: "https://canvas.instructure.com".to_string(),
        }
    }

    pub fn build<Conn: Clone>(self, http: hyper::Client<Conn>) -> Client<Conn> {
        Client {
            hyper: http,
            auth: self.auth,
            base_uri: self.base_url,
        }
    }

    pub fn auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }
    pub fn base_url<U: Into<String>>(mut self, base_url: U) -> Self {
        self.base_url = base_url.into();
        self
    }
}
