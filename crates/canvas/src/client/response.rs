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
            .map_err(|je| Error::from_json_err(je, std::str::from_utf8(&body).unwrap().to_string()))
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
            cost = hyper
                .headers()
                .get("x-request-cost")
                .map(|hv| hv.to_str().unwrap()),
            ratelimit_remaining = hyper
                .headers()
                .get("x-ratelimit-remaining")
                .map(|hv| hv.to_str().unwrap()),
        );
        Response { hyper }
    }
}
