use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::Mutex;
use tokio::task;

use novax::caching::{CachingDurationStrategy, CachingStrategy};
use novax::errors::CachingError;
use novax::errors::NovaXError;

use crate::date::get_current_timestamp::{get_current_timestamp, GetDuration};

#[derive(Clone, Debug)]
pub struct CachingLocal {
    duration_strategy: CachingDurationStrategy,
    value_map: Arc<Mutex<HashMap<u64, Vec<u8>>>>,
    expiration_timestamp_map: Arc<Mutex<HashMap<u64, Duration>>>,
    cleanup_interval: Arc<Mutex<Duration>>,
    is_cleanup_process_started: Arc<Mutex<bool>>,
}

impl CachingLocal {
    pub fn empty(duration_strategy: CachingDurationStrategy) -> CachingLocal {
        CachingLocal {
            duration_strategy,
            value_map: Arc::new(Mutex::new(HashMap::new())),
            expiration_timestamp_map: Arc::new(Mutex::new(HashMap::new())),
            cleanup_interval: Arc::new(Mutex::new(Duration::from_secs(0))),
            is_cleanup_process_started: Arc::new(Mutex::new(false)),
        }
    }

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

    async fn remove_key(&self, key: u64) {
        let _ = self.expiration_timestamp_map.lock().await.remove(&key);
        let _ = self.value_map.lock().await.remove(&key);
    }

    async fn clear(&self) {
        self.expiration_timestamp_map.lock().await.clear();
        self.value_map.lock().await.clear();
    }

    async fn set_value<T: Serialize + DeserializeOwned>(&self, key: u64, value: &T) -> Result<(), NovaXError> {
        let expiration_timestamp = self.duration_strategy.get_duration_timestamp(&get_current_timestamp()?)?;
        self.expiration_timestamp_map.lock().await.insert(key, expiration_timestamp);

        let Ok(serialized) = rmp_serde::to_vec(value) else { return Err(CachingError::UnableToSerialize.into())};
        self.value_map.lock().await.insert(key, serialized);

        Ok(())
    }

    /// Set the cleanup duration for self and all the cloned instances.
    pub async fn set_cleanup_interval(&mut self, interval: Duration) {
        let mut locked = self.cleanup_interval.lock().await;

        *locked = interval;
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
        let current_timestamp = get_current_timestamp()?;
        let mut value_map_locked = self.value_map.lock().await;
        let mut expiration_map_locked = self.expiration_timestamp_map.lock().await;

        for (key, duration) in expiration_map_locked.clone().into_iter() {
            if current_timestamp > duration {
                value_map_locked.remove(&key);
                expiration_map_locked.remove(&key);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl CachingStrategy for CachingLocal {
    async fn get_cache<T: Serialize + DeserializeOwned + Send>(&self, key: u64) -> Result<Option<T>, NovaXError> {
        let Some(expiration_timestamp) = self.expiration_timestamp_map.lock().await.get(&key).cloned() else { return Ok(None) };

        if get_current_timestamp()? >= expiration_timestamp {
            self.remove_key(key).await;
            Ok(None)
        } else {
            let Some(encoded_value) = self.value_map.lock().await.get(&key).cloned() else { return Ok(None) };
            let Ok(value) = rmp_serde::from_slice(&encoded_value) else {
                return Err(CachingError::UnableToDeserialize.into())
            };

            Ok(Some(value))
        }
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
        self.clear().await;

        Ok(())
    }

    fn with_duration_strategy(&self, strategy: CachingDurationStrategy) -> Self {
        CachingLocal {
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
        caching.clear().await;

        assert!(caching.value_map.lock().await.is_empty());
        assert!(caching.expiration_timestamp_map.lock().await.is_empty());

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

        let value_map_locked = caching.value_map.lock().await;
        let expiration_timestamp_locked = caching.expiration_timestamp_map.lock().await;

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

        let value_map_locked = caching.value_map.lock().await;
        let expiration_timestamp_locked = caching.expiration_timestamp_map.lock().await;

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
            let value_map_locked = caching.value_map.lock().await;
            let expiration_timestamp_locked = caching.expiration_timestamp_map.lock().await;

            assert_eq!(value_map_locked.len(), 2);
            assert_eq!(expiration_timestamp_locked.len(), 2);
        }

        caching.perform_cleanup().await?;

        {
            let value_map_locked = caching.value_map.lock().await;
            let expiration_timestamp_locked = caching.expiration_timestamp_map.lock().await;

            assert_eq!(value_map_locked.len(), 1);
            assert_eq!(expiration_timestamp_locked.len(), 1);
        }

        Ok(())
    }
}