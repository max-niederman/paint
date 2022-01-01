use std::ops::Deref;

use super::{pagination::PaginationLinks, Error, Result};
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub struct Response {
    hyper: hyper::Response<hyper::Body>,
}

impl Response {
    #[inline]
    pub async fn deserialize<T: DeserializeOwned>(self) -> Result<T> {
        let body = hyper::body::to_bytes(self.hyper.into_body()).await?;
        serde_json::from_slice(&body)
            .map_err(|je| {
                tracing::warn!(message = "deserialization error", target = std::any::type_name::<T>(), error = %je);
                Error::from_json_err(je, std::str::from_utf8(&body).unwrap().to_string())
            })
    }

    #[inline]
    pub fn pagination_links(&self) -> Result<PaginationLinks> {
        PaginationLinks::from_headers(self.hyper.headers())
    }
}

impl From<hyper::Response<hyper::Body>> for Response {
    #[inline]
    fn from(hyper: hyper::Response<hyper::Body>) -> Self {
        Response { hyper }
    }
}

impl Deref for Response {
    type Target = hyper::Response<hyper::Body>;
    fn deref(&self) -> &Self::Target {
        &self.hyper
    }
}
