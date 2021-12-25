use crate::store::SledStore;
use canvas::{
    client::hyper::{self, client::HttpConnector},
    resource::*,
    DateTime,
};
use ebauche::{
    fetch::{self, Fetch},
    rpc::{
        self,
        message::{DResource, FetchResponse, UpdateResponse},
        *,
    },
};
use futures::{stream, Stream};
use hyper_tls::HttpsConnector;
use miette::miette;
use miette::{Diagnostic, IntoDiagnostic, WrapErr};
use ouroboros::self_referencing;
use pigment::{
    cache::{self, Cache, CacheEntry},
    DSelector, View,
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
                    .view_update::<Course, DSelector>("courses", view, since)]))
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

    pub fn fetch_view<'s, R, RStream>(
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

            Ok(Response::Fetch(FetchResponse::Progress {
                resource: tree_name.to_string(),
            }))
        })
    }

    pub fn view_update<'s, R, S>(
        &'s self,
        tree_name: &'s str,
        view: View,
        since: DateTime,
    ) -> impl Stream<Item = Result<Response, BoxedDiagnostic>> + 's
    where
        R: Cache + Into<DResource> + 's,
    {
        let store = match self
            .db
            .open_tree(tree_name)
            .map(SledStore::from)
            .into_diagnostic()
            .wrap_err("failed to open sled tree")
        {
            Ok(store) => store,
            Err(err) => return ViewUpdate::<_, R>::Error(err.into()),
        };

        match ViewUpdateInner::try_new(view, since, store, cache::get_all) {
            Ok(inner) => ViewUpdate::Ok(inner),
            Err(err) => ViewUpdate::Error(err.into()),
        }
    }
}

#[pin_project(project = ViewUpdateProj)]
enum ViewUpdate<St, R>
where
    St: Stream<Item = pigment::Result<(R::Key, CacheEntry<R>)>>,
    R: Cache,
{
    Ok(#[pin] ViewUpdateInner<St, R>),
    Error(BoxedDiagnostic),
    Finished,
}

#[self_referencing]
struct ViewUpdateInner<St, R>
where
    St: Stream<Item = pigment::Result<(R::Key, CacheEntry<R>)>>,
    R: Cache,
{
    view: View,
    since: DateTime,

    store: SledStore,
    #[borrows(store, view)]
    stream: St,
}

impl<St, R> ViewUpdateInner<St, R>
where
    St: Stream<Item = pigment::Result<(R::Key, CacheEntry<R>)>>,
    R: Cache,
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

impl<St, R> Stream for ViewUpdate<St, R>
where
    St: Stream<Item = pigment::Result<(R::Key, CacheEntry<R>)>>,
    R: Cache + Into<DResource>,
{
    type Item = Result<Response, BoxedDiagnostic>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        // the return value
        let ret = match self.as_mut().project() {
            ViewUpdateProj::Ok(inner) => {
                let since = inner.borrow_since().clone();
                inner
                    .with_stream_mut_pinned(|stream| stream.poll_next(cx))
                    .map(|item| {
                        item.map(|item| match item {
                            Ok((_, entry)) if entry.updated > since => Ok(Response::Update(
                                UpdateResponse::Resource(entry.resource.into()),
                            )),
                            Ok((key, _)) => Ok(Response::Update(UpdateResponse::Stub(
                                cache::key::Key::serialize(&key)?,
                            ))),
                            Err(err) => Err(err.into()),
                        })
                    })
            }
            ViewUpdateProj::Error(err) => Poll::Ready(Some(Err(std::mem::replace(
                err,
                miette!("error already consumed").into(),
            )))),
            ViewUpdateProj::Finished => Poll::Ready(None),
        };

        // transition state
        match *self {
            ViewUpdate::Error(_) => {
                self.set(ViewUpdate::Finished);
                cx.waker().wake_by_ref();
            }
            _ => {}
        }

        ret
    }
}
