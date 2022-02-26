use std::ops::Deref;

use super::{pagination::PaginationLinks, Error, Result};
use hyper::StatusCode;
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
    pub fn throttling(&self) -> Throttling {
        Throttling {
            // TODO: are there any cases where a 403 response isn't throttling?
            throttled: self.hyper.status() == StatusCode::FORBIDDEN,
            cost: self
                .hyper
                .headers()
                .get("X-Request-Cost")
                .and_then(|hv| hv.to_str().ok())
                .and_then(|s| s.parse().ok()),
            remaining: self
                .hyper
                .headers()
                .get("X-Rate-Limit-Remaining")
                .and_then(|hv| hv.to_str().ok())
                .and_then(|s| s.parse().ok()),
        }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Throttling {
    pub throttled: bool,
    pub cost: Option<f64>,
    pub remaining: Option<f64>,
}

macro_rules! response_throttling_getter {
    ($name:ident) => {
        #[inline]
        pub fn $name(&self) -> Result<f64> {
            self.$name
                .ok_or_else(|| Error::MissingThrottlingInfo(stringify!($name)).into())
        }
    };
}

impl Throttling {
    pub const fn throttling(&self) -> bool {
        self.throttled
    }

    response_throttling_getter!(cost);
    response_throttling_getter!(remaining);
}
