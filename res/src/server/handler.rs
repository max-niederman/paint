use std::pin::Pin;

use crate::fetch::Fetch;
use canvas::{
    client::hyper::{self, client::HttpConnector},
    resource::*,
};
use futures::{stream, Stream};
use hyper_tls::HttpsConnector;
use miette::{Diagnostic, IntoDiagnostic, WrapErr};
use pigment::{
    cache,
    rpc::{self, *},
};

#[derive(Clone, Debug)]
pub struct Handler {
    db: sled::Db,
    http_client: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl Handler {
    pub fn new(db: sled::Db) -> Self {
        Self {
            db,
            http_client: hyper::Client::builder().build(HttpsConnector::new()),
        }
    }
}

impl rpc::Handler for Handler {
    type Err = Box<dyn Diagnostic + Send + Sync + 'static>;
    type ResponseStream<'h> = Pin<Box<dyn Stream<Item = Result<Response, Self::Err>> + Send + 'h>>;

    fn handle(&'h self, request: Request) -> Self::ResponseStream<'h> {
        match request {
            Request::Update { view, canvas_token } => {
                log::debug!("updating {} with token {}", view, canvas_token);

                let canvas_client = canvas::Client::<HttpsConnector<HttpConnector>>::builder()
                    .auth(canvas::Auth::Bearer(canvas_token))
                    .base_url(view.truth.base_url.clone())
                    .build(self.http_client.clone());

                Box::pin(stream::once(async move {
                    cache::replace_view(
                        &self
                            .db
                            .open_tree("courses")
                            .into_diagnostic()
                            .wrap_err("failed to open sled tree")?,
                        &view,
                        &mut Course::fetch_all(&canvas_client)?,
                    )
                    .await??;

                    Ok(Response::UpdateFinished)
                }))
            }
            Request::Query { view, selector } => {
                log::debug!("querying {}", view);

                unimplemented!()
            }
        }
    }
}
