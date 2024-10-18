use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::Mutex;

use novax::caching::{CachingDurationStrategy, CachingStrategy};
use novax::errors::CachingError;
use novax::errors::NovaXError;

use crate::date::get_current_timestamp::{get_current_timestamp, GetDuration};

#[derive(Clone, Debug)]
pub struct CachingLocal {
    duration_strategy: CachingDurationStrategy,
    value_map: Arc<Mutex<HashMap<u64, Vec<u8>>>>,
    expiration_timestamp_map: Arc<Mutex<HashMap<u64, Duration>>>
}

impl CachingLocal {
    pub fn empty(duration_strategy: CachingDurationStrategy) -> CachingLocal {
        CachingLocal {
            duration_strategy,
            value_map: Arc::new(Mutex::new(HashMap::new())),
            expiration_timestamp_map: Arc::new(Mutex::new(HashMap::new()))
        }
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

        let Ok(serialized) = bitcode::serialize(value) else { return Err(CachingError::UnableToSerialize.into())};
        self.value_map.lock().await.insert(key, serialized);

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
            let Ok(value) = bitcode::deserialize(&encoded_value) else { return Err(CachingError::UnableToDeserialize.into()) };

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
            expiration_timestamp_map: self.expiration_timestamp_map.clone()
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
}