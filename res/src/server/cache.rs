use crate::store::SledStore;
use canvas::DateTime;
use ebauche::{
    fetch,
    rpc::{message::*, Response},
};
use fallible_stream::YieldError;
use futures::{stream, Stream};
use miette::{Diagnostic, IntoDiagnostic, WrapErr};
use ouroboros::self_referencing;
use pigment::{
    cache::{Cache, CacheEntry},
    View,
};
use std::{
    pin::Pin,
    task::{self, Poll},
};

#[derive(Debug)]
pub struct EbaucheCache {
    db: sled::Db,
}

type BoxedDiagnostic = Box<dyn Diagnostic + Send + Sync + 'static>;

impl EbaucheCache {
    pub fn new(db: sled::Db) -> Self {
        Self { db }
    }

    /// Fetch a view and write it into the cache.
    pub fn fetch_view<'s, R, RStream>(
        &'s self,
        tree_name: &'s str,
        view: View,
        mut resources: RStream,
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

            pigment::cache::replace_view(&store, &view, &mut resources).await??;

            Ok(Response::Fetch(FetchResponse::Progress {
                resource: tree_name.to_string(),
            }))
        })
    }

    /// Get the difference between a view and its past state. 
    pub fn view_diff<'s, R, S>(
        &'s self,
        tree_name: &'s str,
        view: View,
        since: DateTime,
    ) -> YieldError<impl Stream<Item = Result<Response, BoxedDiagnostic>> + 's>
    where
        R: Cache + Into<DResource> + 's,
    {
        let store = self
            .db
            .open_tree(tree_name)
            .map(SledStore::from)
            .into_diagnostic()
            .wrap_err("failed to open sled tree")
            .map_err(BoxedDiagnostic::from)?;

        YieldError::Ok(
            ViewUpdate::<R, _>::try_new(view, since, store, pigment::cache::get_all)
                .map_err(BoxedDiagnostic::from)?,
        )
    }
}

#[self_referencing]
struct ViewUpdate<R, St>
where
    R: Cache,
    St: Stream<Item = pigment::Result<(R::Key, CacheEntry<R>)>>,
{
    view: View,
    since: DateTime,

    store: SledStore,
    #[borrows(store, view)]
    stream: St,
}

impl<R, St> ViewUpdate<R, St>
where
    R: Cache,
    St: Stream<Item = pigment::Result<(R::Key, CacheEntry<R>)>>,
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

impl<R, St> Stream for ViewUpdate<R, St>
where
    R: Cache + Into<DResource>,
    St: Stream<Item = pigment::Result<(R::Key, CacheEntry<R>)>>,
{
    type Item = Result<Response, BoxedDiagnostic>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let since = self.borrow_since().timestamp_millis();
        self.with_stream_mut_pinned(|stream| stream.poll_next(cx))
            .map(|item| {
                item.map(|item| match item {
                    Ok((_, entry)) if entry.updated.timestamp_millis() > since => Ok(
                        Response::Update(UpdateResponse::Resource(entry.resource.into())),
                    ),
                    Ok((key, _)) => Ok(Response::Update(UpdateResponse::Stub(
                        pigment::cache::Key::serialize(&key)?,
                    ))),
                    Err(err) => Err(err.into()),
                })
            })
    }
}
