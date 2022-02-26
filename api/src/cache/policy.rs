//! Cache policies.

use chrono::{Duration, Utc};

use super::{CacheEntry, Resource};

/// Policy for invalidating cache resources.
pub trait InvalidationPolicy<R: Resource>: Send + Sync {
    fn validity(&self, entry: &CacheEntry<R>) -> Validity;
}

/// An action to take with regards to a possibly invalidated cache entry.
#[derive(Debug, Clone, Copy)]
pub enum Validity {
    Invalid,
    Valid,
}

/// A policy which invalidates cache entries exceeding an age limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct MaxAge {
    max_age: Duration,
}

impl<R: Resource> InvalidationPolicy<R> for MaxAge {
    fn validity(&self, entry: &CacheEntry<R>) -> Validity {
        let age = Utc::now() - entry.entered;
        if age > self.max_age {
            Validity::Invalid 
        } else {
            Validity::Valid
        }
    }
}