use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::{Mutex, RwLock};
use tokio::task;

use novax::caching::{CachingDurationStrategy, CachingStrategy};
use novax::errors::CachingError;
use novax::errors::NovaXError;

use crate::date::get_current_timestamp::{get_current_timestamp, GetDuration};
use crate::utils::lock::{Locker, MutexLike};

pub type CachingLocal = BaseCachingLocal<RwLock<Vec<u8>>, RwLock<HashMap<u64, RwLock<Vec<u8>>>>, RwLock<Duration>, RwLock<HashMap<u64, RwLock<Duration>>>, Mutex<Duration>, Mutex<bool>>;

pub struct BaseCachingLocal<LockerValue, LockerValueHashMap, LockerExpiration, LockerExpirationHashMap, MutexCleanupInterval, MutexIsCleanupProcessStarted>
where
    LockerValue: Locker<T = Vec<u8>> + Debug,
    LockerValueHashMap: Locker<T = HashMap<u64, LockerValue>> + Debug,
    LockerExpiration: Locker<T = Duration> + Debug,
    LockerExpirationHashMap: Locker<T = HashMap<u64, LockerExpiration>> + Debug,
    MutexCleanupInterval: MutexLike<T = Duration> + Debug,
    MutexIsCleanupProcessStarted: MutexLike<T = bool> + Debug
{
    duration_strategy: CachingDurationStrategy,
    value_map: Arc<LockerValueHashMap>,
    expiration_timestamp_map: Arc<LockerExpirationHashMap>,
    cleanup_interval: Arc<MutexCleanupInterval>,
    is_cleanup_process_started: Arc<MutexIsCleanupProcessStarted>,
}

impl<LockerValue, LockerValueHashMap, LockerExpiration, LockerExpirationHashMap, MutexCleanupInterval, MutexIsCleanupProcessStarted> Clone for BaseCachingLocal<LockerValue, LockerValueHashMap, LockerExpiration, LockerExpirationHashMap, MutexCleanupInterval, MutexIsCleanupProcessStarted>
where
    LockerValue: Locker<T = Vec<u8>> + Debug,
    LockerValueHashMap: Locker<T = HashMap<u64, LockerValue>> + Debug,
    LockerExpiration: Locker<T = Duration> + Debug,
    LockerExpirationHashMap: Locker<T = HashMap<u64, LockerExpiration>> + Debug,
    MutexCleanupInterval: MutexLike<T = Duration> + Debug,
    MutexIsCleanupProcessStarted: MutexLike<T = bool> + Debug
{
    fn clone(&self) -> Self {
        Self {
            duration_strategy: self.duration_strategy.clone(),
            value_map: self.value_map.clone(),
            expiration_timestamp_map: self.expiration_timestamp_map.clone(),
            cleanup_interval: self.cleanup_interval.clone(),
            is_cleanup_process_started: self.is_cleanup_process_started.clone(),
        }
    }
}

impl<LockerValue, LockerValueHashMap, LockerExpiration, LockerExpirationHashMap, MutexCleanupInterval, MutexIsCleanupProcessStarted> Debug for BaseCachingLocal<LockerValue, LockerValueHashMap, LockerExpiration, LockerExpirationHashMap, MutexCleanupInterval, MutexIsCleanupProcessStarted>
where
    LockerValue: Locker<T = Vec<u8>> + Debug,
    LockerValueHashMap: Locker<T = HashMap<u64, LockerValue>> + Debug,
    LockerExpiration: Locker<T = Duration> + Debug,
    LockerExpirationHashMap: Locker<T = HashMap<u64, LockerExpiration>> + Debug,
    MutexCleanupInterval: MutexLike<T = Duration> + Debug,
    MutexIsCleanupProcessStarted: MutexLike<T = bool> + Debug
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseCachingLocal")
            .field("duration_strategy", &self.duration_strategy)
            .field("value_map", &self.value_map)
            .field("expiration_timestamp_map", &self.expiration_timestamp_map)
            .field("cleanup_interval", &self.cleanup_interval)
            .field("is_cleanup_process_started", &self.is_cleanup_process_started)
            .finish()
    }
}

impl<LockerValue, LockerValueHashMap, LockerExpiration, LockerExpirationHashMap, MutexCleanupInterval, MutexIsCleanupProcessStarted> BaseCachingLocal<LockerValue, LockerValueHashMap, LockerExpiration, LockerExpirationHashMap, MutexCleanupInterval, MutexIsCleanupProcessStarted>
where
    LockerValue: Locker<T = Vec<u8>> + Debug,
    LockerValueHashMap: Locker<T = HashMap<u64, LockerValue>> + Debug,
    LockerExpiration: Locker<T = Duration> + Debug,
    LockerExpirationHashMap: Locker<T = HashMap<u64, LockerExpiration>> + Debug,
    MutexCleanupInterval: MutexLike<T = Duration> + Debug,
    MutexIsCleanupProcessStarted: MutexLike<T = bool> + Debug
{
    pub fn empty(duration_strategy: CachingDurationStrategy) -> Self {
        BaseCachingLocal {
            duration_strategy,
            value_map: Arc::new(LockerValueHashMap::new(HashMap::new())),
            expiration_timestamp_map: Arc::new(LockerExpirationHashMap::new(HashMap::new())),
            cleanup_interval: Arc::new(MutexCleanupInterval::new(Duration::from_secs(0))),
            is_cleanup_process_started: Arc::new(MutexIsCleanupProcessStarted::new(false)),
        }
    }

    async fn remove_key(&self, key: u64) {
        let contains_key = {
            let expiration_timestamp_read_guard = self.expiration_timestamp_map.read().await;
            expiration_timestamp_read_guard.contains_key(&key)
        };

        if contains_key {
            let mut expiration_write_guard = self.expiration_timestamp_map.write().await;
            let mut value_map_write_guard = self.value_map.write().await;

            expiration_write_guard.remove(&key);
            value_map_write_guard.remove(&key);
        }
    }

    async fn set_value<T: Serialize + DeserializeOwned>(&self, key: u64, value: &T) -> Result<(), NovaXError> {
        let contains_key = {
            let expiration_timestamp_read_guard = self.expiration_timestamp_map.read().await;
            expiration_timestamp_read_guard.contains_key(&key)
        };
        
        let expiration_timestamp = self.duration_strategy.get_duration_timestamp(&get_current_timestamp()?)?;
        let Ok(serialized) = rmp_serde::to_vec(value) else { return Err(CachingError::UnableToSerialize.into())};
        
        if contains_key {
            let mut expiration_timestamp_map_read_guard = self.expiration_timestamp_map.read().await;
            // Important: the key might have been removed since the contains_key assignment.
            // If so, we won't set the cache here, but go to the "!contains_key" scope.
            // We could lock the whole map but this has a terrible performance impact by creating a bottleneck.
            if let Some(expiration_timestamp_locker) = expiration_timestamp_map_read_guard.get(&key) {
                let mut expiration_timestamp_write = expiration_timestamp_locker.write().await;

                // Let's do the same for the value
                let mut value_map_read_guard = self.value_map.read().await;
                if let Some(value_locker) = value_map_read_guard.get(&key) {
                    let mut value_write = value_locker.write().await;
                    *expiration_timestamp_write = expiration_timestamp;
                    *value_write = serialized;

                    return Ok(());
                };
            };
        }

        // The key is not found, we have to lock everything.
        let mut expiration_map_write_guard = self.expiration_timestamp_map.write().await;
        let mut value_map_write_guard = self.value_map.write().await;
        expiration_map_write_guard.insert(key, LockerExpiration::new(expiration_timestamp));
        value_map_write_guard.insert(key, LockerValue::new(serialized));

        Ok(())
    }

    /// Set the cleanup duration for self and all the cloned instances.
    pub async fn set_cleanup_interval(&mut self, interval: Duration) {
        let mut locked = self.cleanup_interval.lock().await;

        *locked = interval;
    }
}

impl CachingLocal
{
    pub async fn empty_with_auto_cleanup(
        duration_strategy: CachingDurationStrategy,
        cleanup_interval: Duration
    ) -> Result<CachingLocal, NovaXError> {
        let caching = CachingLocal::empty(duration_strategy);

        {
            let mut locked = caching.cleanup_interval.lock().await;
            *locked = cleanup_interval;
        }

        caching.start_cleanup_process_if_needed().await;

        Ok(caching)
    }

    async fn start_cleanup_process_if_needed(&self) {
        {
            let mut locked = self.is_cleanup_process_started.lock().await;

            if *locked {
                return;
            }

            *locked = true;
        }

        let self_value = self.clone();

        task::spawn(async move {
            loop {
                let duration = {
                    let locked = self_value.cleanup_interval.lock().await;

                    *locked
                };

                let wait_duration = if duration.is_zero() {
                    Duration::from_secs(10)
                } else {
                    self_value.perform_cleanup().await?;

                    duration
                };

                tokio::time::sleep(wait_duration).await
            }

            #[allow(unreachable_code)]
            Ok::<_, NovaXError>(())
        });
    }

    async fn perform_cleanup(&self) -> Result<(), NovaXError> {
        // Can create a bottleneck, be sure to not run this function too frequently.
        let current_timestamp = get_current_timestamp()?;
        let mut expiration_map_write_guard = self.expiration_timestamp_map.write().await;
        let mut value_map_write_guard = self.value_map.write().await;

        let keys: Vec<u64> = expiration_map_write_guard
            .keys()
            .clone()
            .into_iter()
            .map(|e| *e)
            .collect();

        for key in keys {
            let should_remove = {
                let Some(duration_locker) = expiration_map_write_guard.get(&key) else {
                    continue;
                };

                let duration_read = duration_locker.read().await;

                current_timestamp > *duration_read
            };

            if should_remove {
                value_map_write_guard.remove(&key);
                expiration_map_write_guard.remove(&key);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<LockerValue, LockerValueHashMap, LockerExpiration, LockerExpirationHashMap, MutexCleanupInterval, MutexIsCleanupProcessStarted> CachingStrategy for BaseCachingLocal<LockerValue, LockerValueHashMap, LockerExpiration, LockerExpirationHashMap, MutexCleanupInterval, MutexIsCleanupProcessStarted>
where
    LockerValue: Locker<T = Vec<u8>> + Debug,
    LockerValueHashMap: Locker<T = HashMap<u64, LockerValue>> + Debug,
    LockerExpiration: Locker<T = Duration> + Debug,
    LockerExpirationHashMap: Locker<T = HashMap<u64, LockerExpiration>> + Debug,
    MutexCleanupInterval: MutexLike<T = Duration> + Debug,
    MutexIsCleanupProcessStarted: MutexLike<T = bool> + Debug
{
    async fn get_cache<T: Serialize + DeserializeOwned + Send>(&self, key: u64) -> Result<Option<T>, NovaXError> {
        {
            let expiration_timestamp = {
                let read_guard = self.expiration_timestamp_map.read().await;
                let Some(expiration_timestamp_locker) = read_guard.get(&key) else {
                    return Ok(None);
                };

                let expiration_timestamp_read = expiration_timestamp_locker.read().await;
                *expiration_timestamp_read
            };

            if get_current_timestamp()? >= expiration_timestamp {
                self.remove_key(key).await;
                return Ok(None)
            }
        };

        let value_map_read_guard = self.value_map.read().await;
        let Some(encoded_value_locked) = value_map_read_guard.get(&key) else {
            return Ok(None);
        };

        let encoded_value = encoded_value_locked.read().await;

        let Ok(value) = rmp_serde::from_slice(&encoded_value) else {
            return Err(CachingError::UnableToDeserialize.into())
        };

        Ok(Some(value))
    }

    async fn set_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64, value: &T) -> Result<(), NovaXError> {
        Ok(self.set_value(key, value).await?)
    }

    async fn get_or_set_cache<T, FutureGetter, Error>(&self, key: u64, getter: FutureGetter) -> Result<T, Error>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        FutureGetter: Future<Output=Result<T, Error>> + Send,
        Error: From<NovaXError>
    {
        if let Some(cached_value) = self.get_cache(key).await? {
            Ok(cached_value)
        } else {
            let value = getter.await?;
            self.set_cache(key, &value).await?;
            Ok(value)
        }
    }

    async fn clear(&self) -> Result<(), NovaXError> {
        let mut expiration_map_write_guard = self.expiration_timestamp_map.write().await;
        let mut value_map_write_guard = self.value_map.write().await;

        expiration_map_write_guard.clear();
        value_map_write_guard.clear();

        Ok(())
    }

    fn with_duration_strategy(&self, strategy: CachingDurationStrategy) -> Self {
        Self {
            duration_strategy: strategy,
            value_map: self.value_map.clone(),
            expiration_timestamp_map: self.expiration_timestamp_map.clone(),
            cleanup_interval: self.cleanup_interval.clone(),
            is_cleanup_process_started: self.is_cleanup_process_started.clone()
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use novax::caching::{CachingDurationStrategy, CachingStrategy};
    use novax::errors::NovaXError;

    use crate::date::get_current_timestamp::set_mock_time;
    use crate::local::caching_local::CachingLocal;
    use crate::utils::lock::Locker;

    #[tokio::test]
    async fn test_get_cache_key_not_found() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let key = 1;

        let result = caching.get_cache::<()>(key).await?;

        assert_eq!(result, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_set_cache() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        let result = caching.get_cache::<String>(key).await?;
        let expected = Some("test".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_before_expiration() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty(CachingDurationStrategy::Duration(Duration::from_secs(10)));
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(9));

        let result = caching.get_cache::<String>(key).await?;
        let expected = Some("test".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_after_expiration() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty(CachingDurationStrategy::Duration(Duration::from_secs(10)));
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(11));

        let result = caching.get_cache::<String>(key).await?;
        let expected = None;

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_start_of_block() -> Result<(), NovaXError> {
        set_mock_time(Duration::from_secs(0));
        let caching = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(5));

        let result = caching.get_cache::<String>(key).await?;
        let expected = Some("test".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_same_block() -> Result<(), NovaXError> {
        set_mock_time(Duration::from_secs(3));
        let caching = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(5));

        let result = caching.get_cache::<String>(key).await?;
        let expected = Some("test".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_next_block() -> Result<(), NovaXError> {
        set_mock_time(Duration::from_secs(3));
        let caching = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(6));

        let result = caching.get_cache::<String>(key).await?;
        let expected = None;

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_set_cache_without_previous_value() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let key = 1;

        let result = caching.get_or_set_cache(
            key,
            async {
                Ok::<_, NovaXError>("test".to_string())
            }
        ).await?;

        let expected = "test".to_string();

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_set_cache_with_previous_value() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let key = 1;

        caching.set_cache(key, &"old value".to_string()).await?;

        let result = caching.get_or_set_cache(
            key,
            async {
                Ok::<_, NovaXError>("test".to_string())
            }
        ).await?;

        let expected = "old value".to_string();

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_set_cache_with_previous_value_after_expiration() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty(CachingDurationStrategy::Duration(Duration::from_secs(10)));
        let key = 1;

        caching.set_cache(key, &"old value".to_string()).await?;

        set_mock_time(Duration::from_secs(11));

        let result = caching.get_or_set_cache(
            key,
            async {
                Ok::<_, NovaXError>("test".to_string())
            }
        ).await?;

        let expected = "test".to_string();

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_clear() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty(CachingDurationStrategy::EachBlock);

        caching.set_cache(1, &"test".to_string()).await?;
        caching.set_cache(2, &"test2".to_string()).await?;
        caching.clear().await?;

        assert!(caching.value_map.write().await.is_empty());
        assert!(caching.expiration_timestamp_map.write().await.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_perform_cleanup_before_expiration() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty(CachingDurationStrategy::Duration(Duration::from_secs(10)));
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(10));

        caching.perform_cleanup().await?;

        let value_map_locked = caching.value_map.write().await;
        let expiration_timestamp_locked = caching.expiration_timestamp_map.write().await;

        assert_eq!(value_map_locked.len(), 1);
        assert_eq!(expiration_timestamp_locked.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_perform_cleanup_after_expiration() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty(CachingDurationStrategy::Duration(Duration::from_secs(10)));
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(11));

        caching.perform_cleanup().await?;

        let value_map_locked = caching.value_map.write().await;
        let expiration_timestamp_locked = caching.expiration_timestamp_map.write().await;

        assert!(value_map_locked.is_empty());
        assert!(expiration_timestamp_locked.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_perform_cleanup_one_before_and_after_expiration() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty(CachingDurationStrategy::Duration(Duration::from_secs(10)));
        let key_long_duration = 1;
        let value_long_duration = "test1".to_string();

        caching
            .with_duration_strategy(CachingDurationStrategy::Duration(Duration::from_secs(100)))
            .set_cache(key_long_duration, &value_long_duration)
            .await?;

        let key_short_duration = 2;
        let value_short_duration = "test2".to_string();

        caching
            .with_duration_strategy(CachingDurationStrategy::Duration(Duration::from_secs(10)))
            .set_cache(key_short_duration, &value_short_duration)
            .await?;

        set_mock_time(Duration::from_secs(11));

        {
            let value_map_locked = caching.value_map.write().await;
            let expiration_timestamp_locked = caching.expiration_timestamp_map.write().await;

            assert_eq!(value_map_locked.len(), 2);
            assert_eq!(expiration_timestamp_locked.len(), 2);
        }

        caching.perform_cleanup().await?;

        {
            let value_map_locked = caching.value_map.write().await;
            let expiration_timestamp_locked = caching.expiration_timestamp_map.write().await;

            assert_eq!(value_map_locked.len(), 1);
            assert_eq!(expiration_timestamp_locked.len(), 1);
        }

        Ok(())
    }
}