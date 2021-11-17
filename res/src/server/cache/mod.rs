//! Caching and Querying Schemes for Canvas Resources

// TODO: implement shared API

pub trait Cache: Resource {}

pub struct CacheEntry<R: Resource> {
    resource: R,
}
