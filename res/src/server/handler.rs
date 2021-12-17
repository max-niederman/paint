use std::pin::Pin;

use crate::{fetch::Fetch, store::SledStore};
use canvas::{
    client::hyper::{self, client::HttpConnector},
    resource::*,
};
use ebauche::{
    cache::{self, Cache},
    Selector, View,
};
use futures::{stream, Stream};
use hyper_tls::HttpsConnector;
use miette::{Diagnostic, IntoDiagnostic, WrapErr};
use pigment::rpc::{self, *};

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

type BoxedDiagnostic = Box<dyn Diagnostic + Send + Sync + 'static>;

impl<'h> rpc::Handler<'h> for Handler {
    type Err = BoxedDiagnostic;
    type ResponseStream = Pin<Box<dyn Stream<Item = Result<Response, Self::Err>> + Send + 'h>>;

    fn handle(&'h self, request: Request) -> Self::ResponseStream {
        match request {
            Request::Update { view, canvas_token } => {
                log::debug!("updating {} with token {}", view, canvas_token);

                let canvas_client = canvas::Client::<HttpsConnector<HttpConnector>>::builder()
                    .auth(canvas::Auth::Bearer(canvas_token))
                    .base_url(view.truth.base_url.clone())
                    .build(self.http_client.clone());

                Box::pin(stream::once(async move {
                    let store: SledStore = self
                        .db
                        .open_tree("courses")
                        .into_diagnostic()
                        .wrap_err("failed to open sled tree")?
                        .into();

                    cache::replace_view(&store, &view, &mut Course::fetch_all(&canvas_client)?)
                        .await??;

                    Ok(Response::UpdateFinished)
                }))
            }
            Request::Query { view, selector } => {
                log::debug!("querying {}", view);

                todo!()
            }
        }
    }
}

struct PigmentCache<'db> {
    db: &'db sled::Db,
}

impl<'db> SledCache<'db> {
    pub fn new(db: &'db sled::Db) -> Self {
        Self { db }
    }

    pub fn update_view<
        R: Cache,
        RStream: Stream<Item = Result<R, BoxedDiagnostic>> + Unpin + Send + 'static,
    >(
        &self,
        view: &View,
        resources: &mut RStream,
    ) -> impl Stream<Item = Result<Response, BoxedDiagnostic>> {
        let view = view.clone();
        stream::once(async move {
            let store: SledStore = self
                .db
                .open_tree("courses")
                .into_diagnostic()
                .wrap_err("failed to open sled tree")?
                .into();

            cache::replace_view(&store, &view, resources).await??;

            Ok(Response::UpdateFinished)
        })
    }
}
