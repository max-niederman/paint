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
        tracing::debug!(
            message = "recieved response",
            cost = hyper.headers().get("X-Request-Cost").map(|hv| hv
                .to_str()
                .unwrap()
                .parse::<f64>()
                .unwrap()),
            ratelimit_remaining = hyper.headers().get("X-Rate-Limit-Remaining").map(|hv| hv
                .to_str()
                .unwrap()
                .parse::<f64>()
                .unwrap()),
        );
        Response { hyper }
    }
}
