//! Caching endpoints.

use chrono::{DateTime, Utc};
use poem::{
    web::{Json, Path},
    Endpoint, FromRequest, IntoResponse, Middleware, Request, Response, Result,
};
use policy::{InvalidationPolicy, Validity};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod key;
pub mod policy;
pub mod sled;

pub use key::Key;
use uuid::Uuid;

// TODO: add tracing events

/// A resource which may be cached.
pub trait Resource: Serialize + DeserializeOwned + Send + Sync {
    type Key: Key;

    /// Get the key identifying the resource.
    fn key(&self) -> Self::Key;
}

/// A backing store for a cache.
///
/// A store's clone should always reflect changes in the original store like an [`Arc`] would.
pub trait Store: Clone + Send + Sync {
    type Resource: Resource;

    fn get(
        &self,
        view: Uuid,
        key: &<Self::Resource as Resource>::Key,
    ) -> Result<Option<CacheEntry<Self::Resource>>>;
    fn insert(
        &self,
        view: Uuid,
        resource: &CacheEntry<Self::Resource>,
    ) -> Result<Option<CacheEntry<Self::Resource>>>;
}

/// An entry stored in a cache.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
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
    fn into_response(self) -> Response {
        Json(self.resource)
            .with_header("X-Cache-Entered", self.entered.to_string())
            .into_response()
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
        #[derive(Deserialize)]
        struct ViewPath {
            view: Uuid,
        }
        let ViewPath { view } = Path::<ViewPath>::from_request_without_body(&req).await?.0;

        let key = <S::Resource as Resource>::Key::from_request_without_body(&req).await?;

        match self.cache.store.get(view, &key)? {
            Some(cached) => match self.cache.invalidation.validity(&cached) {
                Validity::Invalid => {
                    let entry = self.endpoint.call(req).await?;
                    self.cache.store.insert(view, &entry)?;
                    Ok(entry)
                }
                Validity::Valid => Ok(cached),
            },
            None => {
                let entry = self.endpoint.call(req).await?;
                self.cache.store.insert(view, &entry)?;
                Ok(entry)
            }
        }
    }
}
