use std::pin::Pin;

use crate::store::SledStore;
use canvas::client::hyper::{self, client::HttpConnector};
use ebauche::{
    cache::{self, Cache},
    View,
};
use futures::{future, stream, Stream, StreamExt};
use hyper_tls::HttpsConnector;
use miette::{Diagnostic, IntoDiagnostic, WrapErr};
use pigment::{
    fetch::{self, Fetch},
    rpc::{self, *},
};

#[derive(Debug)]
pub struct Handler {
    cache: PigmentCache,
    http_client: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl Handler {
    pub fn new(db: sled::Db) -> Self {
        Self {
            cache: PigmentCache::new(db),
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
                tracing::info!(message = "handling update request", %view, %canvas_token);

                let canvas_client = canvas::Client::<HttpsConnector<HttpConnector>>::builder()
                    .auth(canvas::Auth::Bearer(canvas_token))
                    .base_url(view.truth.base_url.clone())
                    .build(self.http_client.clone());

                Box::pin(
                    stream::select_all([Box::pin(self.cache.update_view(
                        "courses",
                        view,
                        canvas_client.fetch_all(),
                    ))])
                    .chain(stream::once(future::ready(Ok(Response::UpdateFinished)))),
                )
            }
            Request::Query { view, selector: _ } => {
                tracing::info!(message = "handling query request", %view);

                todo!()
            }
        }
    }
}

#[derive(Debug)]
struct PigmentCache {
    db: sled::Db,
}

impl PigmentCache {
    pub fn new(db: sled::Db) -> Self {
        Self { db }
    }

    pub fn update_view<
        's,
        R: Cache,
        RStream: Stream<Item = fetch::Result<R>> + Unpin + Send + 'static,
    >(
        &'s self,
        tree_name: &'s str,
        view: View,
        resources: fetch::Result<RStream>,
    ) -> impl Stream<Item = Result<Response, BoxedDiagnostic>> + '_ {
        stream::once(async move {
            let store: SledStore = self
                .db
                .open_tree(tree_name)
                .into_diagnostic()
                .wrap_err("failed to open sled tree")?
                .into();

            cache::replace_view(&store, &view, &mut resources?).await??;

            Ok(Response::UpdateProgress {
                resource: tree_name.to_string(),
            })
        })
    }
}
