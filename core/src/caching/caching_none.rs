use std::future::Future;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::caching::caching_strategy::{CachingDurationStrategy, CachingStrategy};
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

    fn with_duration_strategy(&self, _strategy: CachingDurationStrategy) -> Self {
        CachingNone
    }
}