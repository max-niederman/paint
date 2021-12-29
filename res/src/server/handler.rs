use crate::cache::EbaucheCache;
use canvas::{
    client::hyper::{self, client::HttpConnector},
    resource,
};
use ebauche::{
    fetch::{tiered::TieredFetcher, Fetch},
    rpc::{self, Request, Response},
};
use futures::prelude::*;
use hyper_tls::HttpsConnector;
use miette::GraphicalReportHandler;
use miette::{Diagnostic, ReportHandler};
use pigment::DSelector;
use std::{fmt, pin::Pin};

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

type CanvasClient = canvas::Client<HttpsConnector<HttpConnector>>;

impl<'h> rpc::Handler<'h> for Handler {
    type Err = PrettyBoxedDiagnostic;
    type ResponseStream = Pin<Box<dyn Stream<Item = Result<Response, Self::Err>> + Send + 'h>>;

    #[tracing::instrument(skip(self))]
    fn handle(&'h self, request: Request) -> Self::ResponseStream {
        match request {
            Request::Fetch { view, canvas_token } => {
                tracing::info!(message = "handling fetch request", %view, %canvas_token);

                let canvas_client = CanvasClient::builder()
                    .auth(canvas::Auth::Bearer(canvas_token))
                    .base_url(view.truth.base_url.clone())
                    .build(self.http_client.clone());

                stream::select_all([
                    self.cache
                        .fetch_view(
                            "courses",
                            view.clone(),
                            Fetch::<resource::Course>::fetch_independent(&canvas_client),
                        )
                        .boxed(),
                    self.cache
                        .fetch_view(
                            "assignments",
                            view.clone(),
                            Fetch::<resource::Assignment>::fetch_independent(&TieredFetcher(
                                &canvas_client,
                            )),
                        )
                        .boxed(),
                    self.cache
                        .fetch_view(
                            "submissions",
                            view.clone(),
                            Fetch::<resource::Submission>::fetch_independent(&TieredFetcher(
                                &canvas_client,
                            )),
                        )
                        .boxed(),
                ])
                .map_err(PrettyBoxedDiagnostic::from)
                .boxed()
            }
            Request::Update { view, since } => {
                tracing::info!(message = "handling update request", %view);

                stream::select_all([
                    self.cache
                        .view_update::<resource::Course>("courses", view.clone(), since)
                        .boxed(),
                    self.cache
                        .view_update::<resource::Assignment>("assignments", view.clone(), since)
                        .boxed(),
                    self.cache
                        .view_update::<resource::Submission>("submissions", view.clone(), since)
                        .boxed(),
                ])
                .map_err(PrettyBoxedDiagnostic::from)
                .boxed()
            }
        }
    }
}

pub struct PrettyBoxedDiagnostic(Box<dyn Diagnostic + Send + 'static>);

impl From<Box<dyn Diagnostic + Send + Sync + 'static>> for PrettyBoxedDiagnostic {
    fn from(diagnostic: Box<dyn Diagnostic + Send + Sync + 'static>) -> Self {
        Self(diagnostic)
    }
}

impl fmt::Display for PrettyBoxedDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        GraphicalReportHandler::new().debug(&*self.0, f)
    }
}
