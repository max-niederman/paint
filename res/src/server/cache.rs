use crate::store::SledStore;
use canvas::DateTime;
use ebauche::{
    fetch,
    rpc::{message::*, Response},
};
use fallible_stream::YieldError;
use futures::prelude::*;
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
use tracing::Instrument;

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
        tree_name: &'static str,
        view: View,
        resources: RStream,
    ) -> impl Stream<Item = Result<Response, BoxedDiagnostic>> + 's
    where
        R: Cache,
        RStream: Stream<Item = fetch::Result<R>> + Unpin + Send + 'static,
    {
        let view_display = view.clone();
        stream::once(
            async move {
                let store: SledStore = self
                    .db
                    .open_tree(tree_name)
                    .into_diagnostic()
                    .wrap_err("failed to open sled tree")?
                    .into();

                replace_view::replace_view_batched(
                    &store,
                    &view,
                    &mut resources.map_ok(|r| (r.key(), Some(r))),
                )
                .await??;

                Ok(Response::Fetch(FetchResponse::Progress {
                    resource: tree_name.to_string(),
                }))
            }
            .instrument(tracing::info_span!("fetch_view", tree_name, view = %view_display)),
        )
    }

    /// Get an update for a view.
    pub fn view_update<'s, R>(
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
    St: Stream<Item = pigment::cache::Result<(R::Key, CacheEntry<R>)>>,
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
    St: Stream<Item = pigment::cache::Result<(R::Key, CacheEntry<R>)>>,
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
    St: Stream<Item = pigment::cache::Result<(R::Key, CacheEntry<R>)>>,
{
    type Item = Result<Response, BoxedDiagnostic>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let since = self.borrow_since().timestamp_millis();
        self.with_stream_mut_pinned(|stream| stream.poll_next(cx))
            .map(|item| {
                item.map(|item| match item {
                    Ok((_, entry)) if entry.written.timestamp_millis() > since => Ok(
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

mod replace_view {
    use crate::store::SledStore;
    use canvas::DateTime;
    use futures::prelude::*;
    use pigment::{
        cache::{Cache, CacheEntry, Error, Key, Result},
        View,
    };
    use std::time::SystemTime;

    /// Batched, unordered version of [`pigment::cache::replace_view_ordered`].
    /// Atomicity is invalidated if keys are added to the view while this function is clearing it.
    #[inline]
    pub async fn replace_view_batched<R, RStream, E>(
        store: &SledStore,
        view: &View,
        resources: &mut RStream,
    ) -> Result<Result<(), E>>
    where
        R: Cache,
        RStream: Stream<Item = Result<(R::Key, Option<R>), E>> + Unpin,
    {
        let mut batch = sled::Batch::default();
        while let Some(res) = resources.next().await {
            match res {
                Ok((key, resource)) => {
                    let key_bytes = [view.serialize()?, key.serialize()?].concat();

                    let old = store
                        .get(&key_bytes)
                        .map_err(Error::store)?
                        .map(|bytes| {
                            bincode::deserialize::<CacheEntry<R>>(&bytes)
                                .map_err(Error::Deserialization)
                        })
                        .transpose()?;

                    if let Some(resource) = resource {
                        let now: DateTime = SystemTime::now().into();
                        batch.insert(
                            key_bytes.clone(),
                            bincode::serialize(&CacheEntry {
                                updated: now,
                                written: match old {
                                    Some(CacheEntry {
                                        written,
                                        resource: old_resource,
                                        ..
                                    }) if old_resource == resource => written,
                                    _ => SystemTime::now().into(),
                                },
                                resource,
                            })
                            .map_err(Error::Serialization)?,
                        );
                    } else if let Some(old) = old {
                        batch.insert(
                            key_bytes.clone(),
                            bincode::serialize(&CacheEntry {
                                updated: SystemTime::now().into(),
                                ..old
                            })
                            .map_err(Error::Serialization)?,
                        );
                    }
                }
                Err(e) => return Ok(Err(e)),
            }
        }

        for key in store.scan_prefix(&view.serialize()?).keys() {
            store
                .remove(key.map_err(Error::store)?)
                .map_err(Error::store)?;
        }
        store.apply_batch(batch).map_err(Error::store)?;

        Ok(Ok(()))
    }
}
