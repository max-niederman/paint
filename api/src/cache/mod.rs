//! Caching endpoints.

use chrono::{DateTime, Utc};
use poem::{Endpoint, FromRequest, IntoResponse, Middleware, Request, Result};
use policy::{InvalidationPolicy, Validity};
use serde::{de::DeserializeOwned, Serialize};

pub mod error;
pub mod policy;

// TODO: add tracing events

/// A resource which may be cached.
pub trait Resource: Serialize + DeserializeOwned + Send + Sync {
    type Key: Key;

    /// Get the key identifying the resource.
    fn key(&self) -> Self::Key;
}

/// A key uniquely identifying a resource in a cache.
pub trait Key: Sized + Send + Sync {
    /// The length in bytes of the serialized key.
    const SER_LEN: usize;

    /// Serialize the key into a byte array.
    fn key_serialize(&self) -> Result<[u8; Self::SER_LEN]>;
    /// Deserialize the key from a byte iterator.
    fn key_deserialize<I: IntoIterator<Item = u8>>(&self) -> Result<Self>;
}

/// A backing store for a cache.
///
/// A store's clone should always reflect changes in the original store like an [`Arc`] would.
pub trait Store: Clone + Send + Sync {
    type Resource: Resource;

    fn get(
        &self,
        key: &<Self::Resource as Resource>::Key,
    ) -> Result<Option<CacheEntry<Self::Resource>>>;
    fn insert(
        &self,
        resource: &CacheEntry<Self::Resource>,
    ) -> Result<Option<CacheEntry<Self::Resource>>>;
}

/// An entry stored in a cache.
pub struct CacheEntry<R> {
    resource: R,

    /// The time the entry was entered into the cache.
    entered: DateTime<Utc>,
}

/// Cache middleware.
#[derive(Debug, Clone, Copy)]
pub struct Cache<S, I> {
    store: S,
    invalidation: I,
}

impl<E, S, I> Middleware<E> for Cache<S, I>
where
    Self: Clone,
    E: Endpoint<Output = CacheEntry<S::Resource>>,
    S: Store,
    for<'a> <S::Resource as Resource>::Key: FromRequest<'a>,
    I: InvalidationPolicy<S::Resource>,
{
    type Output = CacheEndpoint<E, S, I>;
    fn transform(&self, ep: E) -> Self::Output {
        CacheEndpoint {
            cache: self.clone(),
            endpoint: ep,
        }
    }
}

impl<R: Resource> IntoResponse for CacheEntry<R> {
    fn into_response(self) -> poem::Response {
        todo!()
    }
}

/// Cache endpoint.
pub struct CacheEndpoint<E, S, I> {
    cache: Cache<S, I>,
    endpoint: E,
}

#[poem::async_trait]
impl<E, S, I> Endpoint for CacheEndpoint<E, S, I>
where
    E: Endpoint<Output = CacheEntry<S::Resource>>,
    S: Store,
    for<'a> <S::Resource as Resource>::Key: FromRequest<'a>,
    I: InvalidationPolicy<S::Resource>,
{
    type Output = CacheEntry<S::Resource>;

    async fn call(&self, req: Request) -> poem::Result<Self::Output> {
        let key = FromRequest::from_request_without_body(&req).await?;
        let cached = self.cache.store.get(&key)?;

        match cached {
            Some(cached) => match self.cache.invalidation.validity(&cached) {
                Validity::Invalid => {
                    let entry = self.endpoint.call(req).await?;
                    self.cache.store.insert(&entry)?;
                    Ok(entry)
                }
                Validity::Valid => Ok(cached),
            },
            None => {
                let entry = self.endpoint.call(req).await?;
                self.cache.store.insert(&entry)?;
                Ok(entry)
            }
        }
    }
}
