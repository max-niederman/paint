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
    cache::{Cache, CacheEntry, Key},
    ResourceKind, View,
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
    pub fn fetch_view<R, RStream>(
        &self,
        resource_kind: ResourceKind,
        view: View,
        resources: RStream,
    ) -> impl Stream<Item = Result<Response, BoxedDiagnostic>> + '_
    where
        R: Cache,
        RStream: Stream<Item = fetch::Result<R>> + Unpin + Send + 'static,
    {
        let view_display = view.clone();
        stream::once(
            async move {
                let store: SledStore = self
                    .db
                    .open_tree(tree_name(resource_kind))
                    .into_diagnostic()
                    .wrap_err("failed to open sled tree")?
                    .into();

                replace_view::replace_view_batched(
                    &store,
                    &view,
                    &mut resources.map_ok(|r| (r.key(), Some(r))),
                )
                .await??;

                Ok(Response::Fetch(FetchResponse::Progress { resource_kind }))
            }
            .instrument(tracing::info_span!("fetch_view", ?resource_kind, view = %view_display)),
        )
    }

    /// Get an update for a view.
    pub fn view_update<'s, R>(
        &'s self,
        resource_kind: ResourceKind,
        view: &View,
        since: DateTime,
    ) -> YieldError<impl Stream<Item = Result<Response, BoxedDiagnostic>> + 's>
    where
        R: Cache + 's,
    {
        let store = self
            .db
            .open_tree(tree_name(resource_kind))
            .map(SledStore::from)
            .into_diagnostic()
            .wrap_err("failed to open sled tree")
            .map_err(BoxedDiagnostic::from)?;

        YieldError::Ok(
            ViewUpdate::<R, _>::try_new(since, store, |store| unsafe {
                // SAFETY: ouroboros (as of v0.14) stores fields in [`Box`]s, so we can safely
                //         interpret the reference as unbounded, since the iterator will always
                //         be dropped with the referenced store.
                //         if that changes, this could easily become UB.
                pigment::cache::get_all(
                    std::mem::transmute::<&SledStore, &'static SledStore>(store),
                    view,
                )
            })
            .map_err(BoxedDiagnostic::from)?,
        )
    }
}

fn tree_name(resource_kind: ResourceKind) -> &'static str {
    match resource_kind {
        ResourceKind::Assignment => "assignments",
        ResourceKind::Course => "courses",
    }
}

#[self_referencing]
struct ViewUpdate<R, I>
where
    R: Cache,
    I: Iterator<Item = pigment::cache::Result<(R::Key, CacheEntry<R>)>>,
{
    since: DateTime,
    store: SledStore,
    #[borrows(store)]
    iter: I,
}

impl<R, I> ViewUpdate<R, I>
where
    R: Cache,
    I: Iterator<Item = pigment::cache::Result<(R::Key, CacheEntry<R>)>>,
{
    fn with_iter_mut_pinned<Ret>(self: Pin<&mut Self>, f: impl FnOnce(Pin<&mut I>) -> Ret) -> Ret {
        unsafe {
            self.get_unchecked_mut()
                .with_iter_mut(|iter| f(Pin::new_unchecked(iter)))
        }
    }
}

impl<R, I> Stream for ViewUpdate<R, I>
where
    R: Cache,
    I: Iterator<Item = pigment::cache::Result<(R::Key, CacheEntry<R>)>> + Unpin,
{
    type Item = Result<Response, BoxedDiagnostic>;
    fn poll_next(self: Pin<&mut Self>, _cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let since = self.borrow_since().timestamp_millis();
        Poll::Ready(
            self.with_iter_mut_pinned(|iter| iter.get_mut().next())
                .map(|item| {
                    item.map_err(BoxedDiagnostic::from)
                        .and_then(|(key, entry)| {
                            Ok(Response::Update(UpdateResponse {
                                key: Key::serialize(&key)?,

                                // TODO: if we want the client to be totally consistent with Ebauche after updates,
                                //       we also need to send `entry.updated` when it's greater than `since` in some way.
                                resource: if entry.written.timestamp_millis() > since {
                                    // TODO: this serializes exactly the same entry we just deserialized.
                                    //       in the future, we should use an iterator that also yields serialized entry.
                                    //       almost all deserialization overhead could also be avoided by using zero-copy deserialization.
                                    Some(bincode::serialize(&entry).into_diagnostic().wrap_err(
                                        "while serializing resource to yield to update",
                                    )?)
                                } else {
                                    None
                                },
                            }))
                        })
                }),
        )
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
