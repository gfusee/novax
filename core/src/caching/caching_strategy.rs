use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::errors::NovaXError;

#[derive(Clone, Debug)]
pub enum CachingDurationStrategy {
    EachBlock,
    Duration(Duration)
}

/// The `CachingStrategy` trait defines the interface for caching strategies used within the "novax" crate.
///
/// This trait provides methods for getting, setting, and managing cache data while interacting with smart contracts.
/// By default, the "novax" crate provides a `CachingNone` implementation which does not perform any caching.
/// Developers seeking more advanced caching strategies are directed to the "novax-caching" crate.
///
/// Implementations of this trait are expected to handle caching in a way that suits the needs of the application,
/// be it through in-memory caching, disk caching, or other forms of caching.
///
/// This trait is utilized within the "novax" crate to enable caching of queries made to smart contracts,
/// improving efficiency and reducing the need for redundant network requests to the blockchain.
///
/// Methods in this trait are asynchronous to accommodate for potential network operations or other asynchronous
/// operations that might be necessary in implementations.
#[async_trait]
pub trait CachingStrategy: Clone + Send + Sync + Debug {
    /// Retrieves a cache entry for the specified key if it exists.
    ///
    /// # Parameters
    /// - `key`: The key identifying the cache entry.
    ///
    /// # Returns
    /// - A `Result` containing either the cached value or an error if the operation fails.
    async fn get_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64) -> Result<Option<T>, NovaXError>;

    /// Sets a cache entry for the specified key.
    ///
    /// # Parameters
    /// - `key`: The key identifying the cache entry.
    /// - `value`: The value to be cached.
    ///
    /// # Returns
    /// - A `Result` indicating success or an error if the operation fails.
    async fn set_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64, value: &T) -> Result<(), NovaXError>;

    /// Retrieves a cache entry for the specified key if it exists, otherwise sets the cache entry using the provided getter function.
    ///
    /// # Parameters
    /// - `key`: The key identifying the cache entry.
    /// - `getter`: An asynchronous function used to obtain the value if it is not already cached.
    ///
    /// # Returns
    /// - A `Result` containing either the fetched or cached value, or an error if the operation fails.
    async fn get_or_set_cache<T, FutureGetter, Error>(&self, key: u64, getter: FutureGetter) -> Result<T, Error>
        where
            T: Serialize + DeserializeOwned + Send + Sync,
            FutureGetter: Future<Output=Result<T, Error>> + Send,
            Error: From<NovaXError>;

    /// Clear the entire cache.
    ///
    /// # Returns
    /// - A `Result` indicating success or an error if the operation fails.
    async fn clear(&self) -> Result<(), NovaXError>;

    /// TODO Creates a new `CachingStrategy` instance with a specified cache duration strategy.
    ///
    /// # Parameters
    /// - `strategy`: The duration for which cache entries should be kept.
    ///
    /// # Returns
    /// - A new `CachingStrategy` instance with the specified cache duration strategy.
    fn with_duration_strategy(&self, strategy: CachingDurationStrategy) -> Self;
}


