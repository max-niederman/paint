use crate::store::SledStore;
use canvas::client::hyper::{self, client::HttpConnector};
use canvas_lms::resource::Course;
use pigment::{
    cache::{self, Cache, CacheEntry},
    DSelector, Selector, View,
};
use futures::{future, stream, Stream, StreamExt};
use hyper_tls::HttpsConnector;
use miette::miette;
use miette::{Diagnostic, IntoDiagnostic, WrapErr};
use ouroboros::self_referencing;
use ebauche::{
    fetch::{self, Fetch},
    rpc::{self, message::DResource, *},
};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{self, Poll},
};

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
            Request::Update { view, canvas_token } => {
                tracing::info!(message = "handling update request", %view, %canvas_token);

                let canvas_client = canvas::Client::<HttpsConnector<HttpConnector>>::builder()
                    .auth(canvas::Auth::Bearer(canvas_token))
                    .base_url(view.truth.base_url.clone())
                    .build(self.http_client.clone());

                Box::pin(
                    stream::select_all([Box::pin(self.cache.update_view(
                        "courses",
                        view.clone(),
                        canvas_client.fetch_all(),
                    ))])
                    .chain(stream::once(future::ready(Ok(Response::UpdateFinished)))),
                )
            }
            Request::Query { view, selector } => {
                tracing::info!(message = "handling query request", %view);

                Box::pin(stream::select_all([self
                    .cache
                    .query_view::<Course, DSelector>("courses", view, selector)]))
            }
        }
    }
}

#[derive(Debug)]
struct EbaucheCache {
    db: sled::Db,
}

impl EbaucheCache {
    pub fn new(db: sled::Db) -> Self {
        Self { db }
    }

    pub fn update_view<'s, R, RStream>(
        &'s self,
        tree_name: &'s str,
        view: View,
        resources: fetch::Result<RStream>,
    ) -> impl Stream<Item = Result<Response, BoxedDiagnostic>> + 's
    where
        R: Cache,
        RStream: Stream<Item = fetch::Result<R>> + Unpin + Send + 'static,
    {
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

    pub fn query_view<'s, R, S>(
        &'s self,
        tree_name: &'s str,
        view: View,
        selector: S,
    ) -> impl Stream<Item = Result<Response, BoxedDiagnostic>> + 's
    where
        R: Cache + Into<DResource> + 's,
        S: Selector<R> + 's,
    {
        let store = match self
            .db
            .open_tree(tree_name)
            .map(SledStore::from)
            .into_diagnostic()
            .wrap_err("failed to open sled tree")
        {
            Ok(store) => store,
            Err(err) => return QueryView::Error(err.into()),
        };

        match QueryViewInner::try_new(view, selector, store, cache::get_all) {
            Ok(inner) => QueryView::Ok(inner),
            Err(err) => QueryView::Error(err.into()),
        }
    }
}

#[pin_project(project = QueryViewProj)]
enum QueryView<St, R, Se>
where
    St: Stream<Item = pigment::Result<(R::Key, CacheEntry<R>)>>,
    R: Cache,
    Se: Selector<R>,
{
    Ok(#[pin] QueryViewInner<St, R, Se>),
    Error(BoxedDiagnostic),
    Finished,
}

#[allow(dead_code)]
#[self_referencing]
struct QueryViewInner<St, R, Se>
where
    St: Stream<Item = pigment::Result<(R::Key, CacheEntry<R>)>>,
    R: Cache,
    Se: Selector<R>,
{
    view: View,
    selector: Se,

    store: SledStore,
    #[borrows(store, view)]
    stream: St,
}

impl<St, R, Se> QueryViewInner<St, R, Se>
where
    St: Stream<Item = pigment::Result<(R::Key, CacheEntry<R>)>>,
    R: Cache,
    Se: Selector<R>,
{
    fn with_stream_mut_pinned<Ret, F: FnOnce(Pin<&mut St>) -> Ret>(
        self: Pin<&mut Self>,
        f: F,
    ) -> Ret {
        unsafe {
            self.get_unchecked_mut()
                .with_stream_mut(|unpinned| f(Pin::new_unchecked(unpinned)))
        }
    }
}

impl<St, R, Se> Stream for QueryView<St, R, Se>
where
    St: Stream<Item = pigment::Result<(R::Key, CacheEntry<R>)>>,
    R: Cache + Into<DResource>,
    Se: Selector<R>,
{
    type Item = Result<Response, BoxedDiagnostic>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        // the return value
        let ret = match self.as_mut().project() {
            QueryViewProj::Ok(inner) => inner
                .with_stream_mut_pinned(|stream| stream.poll_next(cx))
                .map(|item| {
                    item.map(|item| {
                        item.map(|(_, v)| Response::Resource(v.resource.into()))
                            .map_err(Into::into)
                    })
                }), // FIXME: select items
            QueryViewProj::Error(err) => Poll::Ready(Some(Err(std::mem::replace(
                err,
                miette!("error already consumed").into(),
            )))),
            QueryViewProj::Finished => Poll::Ready(None),
        };

        // transition state
        match *self {
            QueryView::Error(_) => {
                self.set(QueryView::Finished);
                cx.waker().wake_by_ref();
            }
            _ => {}
        }

        ret
    }
}
