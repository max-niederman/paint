use crate::cache::EbaucheCache;
use canvas::{
    client::hyper::{self, client::HttpConnector},
    resource,
};
use ebauche::{
    fetch::Fetch,
    rpc::{self, Request, Response},
};
use futures::{stream, Stream};
use hyper_tls::HttpsConnector;
use miette::Diagnostic;
use pigment::DSelector;
use std::pin::Pin;

#[derive(Debug)]
pub struct Handler {
    cache: EbaucheCache,
    http_client: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl Handler {
    pub fn new(db: sled::Db) -> Self {
        Self {
            cache: EbaucheCache::new(db),
            http_client: hyper::Client::builder().build(HttpsConnector::new()),
        }
    }
}

type BoxedDiagnostic = Box<dyn Diagnostic + Send + Sync + 'static>;

impl<'h> rpc::Handler<'h> for Handler {
    type Err = BoxedDiagnostic;
    type ResponseStream = Pin<Box<dyn Stream<Item = Result<Response, Self::Err>> + Send + 'h>>;

    #[tracing::instrument(skip(self))]
    fn handle(&'h self, request: Request) -> Self::ResponseStream {
        match request {
            Request::Fetch { view, canvas_token } => {
                tracing::info!(message = "handling fetch request", %view, %canvas_token);

                let canvas_client = canvas::Client::<HttpsConnector<HttpConnector>>::builder()
                    .auth(canvas::Auth::Bearer(canvas_token))
                    .base_url(view.truth.base_url.clone())
                    .build(self.http_client.clone());

                Box::pin(stream::select_all([Box::pin(self.cache.fetch_view(
                    "courses",
                    view.clone(),
                    canvas_client.fetch_all(),
                ))]))
            }
            Request::Update { view, since } => {
                tracing::info!(message = "handling update request", %view);

                Box::pin(stream::select_all([self
                    .cache
                    .view_update::<resource::Course, DSelector>(
                        "courses", view, since,
                    )]))
            }
        }
    }
}
