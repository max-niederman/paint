use reqwest::{IntoUrl, Method, RequestBuilder};

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
                .expect("failed to instantiate HTTP client"),
            base_url: base_url.as_ref().to_owned(),
        }
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
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
    fn build(self) -> Client {
        Client::new(self.base_url, self.auth)
    }

    fn with_base_url<U: AsRef<str>>(&mut self, base_url: U) -> &mut Self {
        self.base_url = base_url.as_ref().to_owned();
        self
    }

    fn with_auth(&mut self, auth: Auth) -> &mut Self {
        self.auth = Some(auth);
        self
    }
}
