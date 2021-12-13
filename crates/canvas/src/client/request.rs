use super::{pagination::Pagination, Auth, Client, Error, Response, Result};
use hyper::{
    client::connect::Connect, header, http::request::Builder as HyperRequestBuilder, Body, Method,
    Request,
};
use std::fmt::Write;

#[derive(Debug)]
pub struct RequestBuilder<'c, Conn: Clone> {
    hyper: HyperRequestBuilder,
    client: &'c Client<Conn>,

    path: String,
    query: Vec<(String, String)>,

    include: Vec<&'static str>,
}

impl<'c, Conn: Clone> RequestBuilder<'c, Conn> {
    pub fn new(client: &'c Client<Conn>, method: Method, path: String) -> Self {
        Self {
            hyper: {
                let mut builder = hyper::Request::builder()
                    .method(method)
                    .header(header::ACCEPT, "application/json");

                match &client.auth {
                    Some(Auth::Bearer(token)) => {
                        builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token))
                    }
                    _ => {}
                }

                builder
            },

            client,
            path,

            query: Vec::new(),

            include: Vec::new(),
        }
    }

    fn build(self, body: Body) -> Result<(&'c Client<Conn>, Request<Body>)> {
        let mut path_and_query = self.path;

        macro_rules! append_query {
            ($($arg:tt)*) => {
                if path_and_query.is_empty() {
                    path_and_query.push('?');
                } else {
                    path_and_query.push('&');
                }

                write!(path_and_query, $($arg)*).unwrap();
            };
        }

        for include in self.include {
            append_query!("include[]={}", include);
        }

        for (key, value) in self.query {
            append_query!("{}={}", key, value);
        }

        self.hyper
            .uri(
                format!(
                    "{base}{path_and_query}",
                    base = self.client.base_uri,
                    path_and_query = path_and_query,
                )
                .as_str(),
            )
            .body(body)
            .map(|req| (self.client, req))
            .map_err(Into::into)
    }

    pub async fn send(self, body: Body) -> Result<Response>
    where
        Conn: Connect + Send + Sync + 'static,
    {
        self.client
            .hyper
            .request(self.build(body)?.1)
            .await
            .map(Response::from)
            .map_err(Error::from)
    }

    pub fn paginate(self) -> Result<Pagination<'c, Conn>>
    where
        Conn: Connect + Clone + Send + Sync + 'static,
    {
        let (client, req) = self.build(Body::empty())?;
        Pagination::new(client, req.headers().clone(), req.uri().clone())
    }
}

impl AsRef<HyperRequestBuilder> for RequestBuilder<'_, ()> {
    fn as_ref(&self) -> &HyperRequestBuilder {
        &self.hyper
    }
}

impl AsMut<HyperRequestBuilder> for RequestBuilder<'_, ()> {
    fn as_mut(&mut self) -> &mut HyperRequestBuilder {
        &mut self.hyper
    }
}
