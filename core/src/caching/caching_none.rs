use async_trait::async_trait;
use std::future::Future;
use std::time::Duration;
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::caching::caching_strategy::CachingStrategy;
use crate::errors::NovaXError;

/// An implementation of the `CachingStrategy` trait that performs no caching.
/// This is the default caching strategy provided by the "novax" crate.
/// Developers looking for more sophisticated caching solutions should refer to the "novax-caching" crate.
#[derive(Clone, Debug)]
pub struct CachingNone;

#[async_trait]
impl CachingStrategy for CachingNone {
    /// Attempts to retrieve a cached value based on a key, but always returns `None`
    /// since `CachingNone` does not perform any caching.
    async fn get_cache<T: Serialize + DeserializeOwned + Send>(&self, _key: u64) -> Result<Option<T>, NovaXError> {
        Ok(None)
    }

    /// Attempts to set a cache value based on a key, but does nothing
    /// since `CachingNone` does not perform any caching.
    async fn set_cache<T: Serialize + DeserializeOwned + Send>(&self, _key: u64, _value: &T) -> Result<(), NovaXError> {
        Ok(())
    }

    /// Attempts to clear the cache, but does nothing
    /// since `CachingNone` does not perform any caching.
    async fn clear(&self) -> Result<(), NovaXError> {
        Ok(())
    }

    /// Either retrieves a cached value or sets a new cache value based on a key,
    /// but simply calls the provided value function since `CachingNone` does not perform any caching.
    async fn get_or_set_cache<T, FutureGetter, Error>(&self, _key: u64, value_fn: FutureGetter) -> Result<T, Error>
        where
            T: Serialize + DeserializeOwned + Send,
            FutureGetter: Future<Output=Result<T, Error>> + Send,
            Error: From<NovaXError>
    {
        value_fn.await
    }

    /// Attempts to create a new `CachingNone` instance with a specified cache duration,
    /// but returns a new unchanged `CachingNone` instance since `CachingNone` does not support cache duration.
    fn with_duration(&self, _duration: Duration) -> Self {
        CachingNone
    }

    /// Attempts to create a new `CachingNone` instance that caches values until the next block,
    /// but returns a new unchanged `CachingNone` instance since `CachingNone` does not support caching until the next block.
    fn until_next_block(&self) -> Self {
        CachingNone
    }
}