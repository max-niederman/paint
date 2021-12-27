use super::{pagination::Pagination, Auth, Client, Error, Response, Result};
use hyper::{
    client::connect::Connect, header, http::request::Builder as HyperRequestBuilder, Body, Method,
    Request,
};
use std::{borrow::Cow, fmt::Write};

#[derive(Debug)]
pub struct RequestBuilder<'c, Conn: Clone> {
    hyper: HyperRequestBuilder,
    client: &'c Client<Conn>,

    path: String,
    query: Vec<(String, String)>,

    include: Vec<&'static str>,
}

impl<'c, Conn: Clone> RequestBuilder<'c, Conn> {
    #[inline]
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

    #[inline]
    fn build(self, body: Body) -> Result<(&'c Client<Conn>, Request<Body>)> {
        let path_len = self.path.len();
        let mut path_and_query = self.path;

        macro_rules! append_query {
            ($($arg:tt)*) => {
                if path_and_query.len() == path_len {
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

    #[inline]
    pub async fn send(self, body: Body) -> Result<Response>
    where
        Conn: Connect + Send + Sync + 'static,
    {
        self.client
            .http
            .request(self.build(body)?.1)
            .await
            .map(Response::from)
            .map_err(Error::from)
    }

    #[inline]
    pub fn paginate(self, per_page: usize) -> Result<Pagination<'c, Conn>>
    where
        Conn: Connect + Clone + Send + Sync + 'static,
    {
        let (client, req) = self
            .query("per_page", per_page.to_string())
            .build(Body::empty())?;
        Pagination::new(
            Cow::Borrowed(client),
            req.headers().clone(),
            req.uri().clone(),
        )
    }

    #[inline]
    pub fn paginate_owned<'a>(self, per_page: usize) -> Result<Pagination<'a, Conn>>
    where
        Conn: Connect + Clone + Send + Sync + 'static,
    {
        let (client, req) = self
            .query("per_page", per_page.to_string())
            .build(Body::empty())?;
        Pagination::new(
            Cow::Owned(client.clone()),
            req.headers().clone(),
            req.uri().clone(),
        )
    }

    #[inline]
    pub fn query<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.query.push((key.into(), value.into()));
        self
    }

    #[inline]
    pub fn extend_query<K, V, I>(mut self, iter: I) -> Self
    where
        K: Into<String>,
        V: Into<String>,
        I: IntoIterator<Item = (K, V)>,
    {
        self.query.extend(
            iter.into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );
        self
    }

    #[inline]
    pub fn include(mut self, val: &'static str) -> Self {
        self.include.push(val);
        self
    }

    #[inline]
    pub fn extend_include<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = &'static str>,
    {
        self.include.extend(iter.into_iter());
        self
    }
}

impl AsRef<HyperRequestBuilder> for RequestBuilder<'_, ()> {
    #[inline]
    fn as_ref(&self) -> &HyperRequestBuilder {
        &self.hyper
    }
}

impl AsMut<HyperRequestBuilder> for RequestBuilder<'_, ()> {
    #[inline]
    fn as_mut(&mut self) -> &mut HyperRequestBuilder {
        &mut self.hyper
    }
}
