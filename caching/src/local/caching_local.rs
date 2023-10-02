use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use std::future::Future;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use novax::caching::CachingStrategy;
use novax::errors::NovaXError;
use novax::errors::CachingError;
use crate::date::get_current_timestamp::get_current_timestamp;

#[derive(Clone, Debug)]
pub struct CachingLocal {
    duration: u64,
    until_next_block: bool,
    value_map: Arc<Mutex<HashMap<u64, Vec<u8>>>>,
    expiration_timestamp_map: Arc<Mutex<HashMap<u64, u64>>>
}

impl CachingLocal {
    pub fn empty() -> CachingLocal {
        CachingLocal {
            duration: 0,
            until_next_block: false,
            value_map: Arc::new(Mutex::new(HashMap::new())),
            expiration_timestamp_map: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    async fn remove_key(&self, key: u64) {
        let _ = self.expiration_timestamp_map.lock().await.remove(&key);
        let _ = self.value_map.lock().await.remove(&key);
    }

    async fn set_value<T: Serialize + DeserializeOwned>(&self, key: u64, value: &T) -> Result<(), NovaXError> {
        let current_timestamp = get_current_timestamp()?;
        let expiration_timestamp = if self.until_next_block {
            let mut timestamp = current_timestamp + 1;
            while timestamp % 6 != 5 {
                timestamp += 1
            }

            timestamp
        } else {
            current_timestamp + self.duration
        };
        self.expiration_timestamp_map.lock().await.insert(key, expiration_timestamp);

        let Ok(serialized) = rmp_serde::to_vec(value) else { return Err(CachingError::UnableToSerialize.into())};
        self.value_map.lock().await.insert(key, serialized);

        Ok(())
    }
}

#[async_trait]
impl CachingStrategy for CachingLocal {
    async fn get_cache<T: Serialize + DeserializeOwned + Send>(&self, key: u64) -> Result<Option<T>, NovaXError> {
        let Some(expiration_timestamp) = self.expiration_timestamp_map.lock().await.get(&key).cloned() else { return Ok(None) };

        if get_current_timestamp()? > expiration_timestamp {
            self.remove_key(key).await;
            Ok(None)
        } else {
            let Some(encoded_value) = self.value_map.lock().await.get(&key).cloned() else { return Ok(None) };
            let Ok(value) = rmp_serde::from_slice(&encoded_value) else { return Err(CachingError::UnableToDeserialize.into()) };

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

    fn with_duration(&self, duration: u64) -> Self {
        CachingLocal {
            duration,
            until_next_block: self.until_next_block,
            value_map: self.value_map.clone(),
            expiration_timestamp_map: self.expiration_timestamp_map.clone()
        }
    }

    fn until_next_block(&self) -> Self {
        CachingLocal {
            duration: 0,
            until_next_block: true,
            value_map: self.value_map.clone(),
            expiration_timestamp_map: self.expiration_timestamp_map.clone()
        }
    }
}

#[cfg(test)]
mod test {
    use novax::caching::CachingStrategy;
    use novax::errors::NovaXError;
    use crate::date::get_current_timestamp::set_mock_time;
    use crate::local::caching_local::CachingLocal;

    #[tokio::test]
    async fn test_get_cache_key_not_found() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty();
        let key = 1;

        let result = caching.get_cache::<()>(key).await?;

        assert_eq!(result, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_set_cache() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty();
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
        let caching = CachingLocal::empty().with_duration(10);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(10);

        let result = caching.get_cache::<String>(key).await?;
        let expected = Some("test".to_string());


        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_after_expiration() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty().with_duration(10);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(11);

        let result = caching.get_cache::<String>(key).await?;
        let expected = None;

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_start_of_block() -> Result<(), NovaXError> {
        set_mock_time(0);
        let caching = CachingLocal::empty().until_next_block();
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(5);

        let result = caching.get_cache::<String>(key).await?;
        let expected = Some("test".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_same_block() -> Result<(), NovaXError> {
        set_mock_time(3);
        let caching = CachingLocal::empty().until_next_block();
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(5);

        let result = caching.get_cache::<String>(key).await?;
        let expected = Some("test".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_next_block() -> Result<(), NovaXError> {
        set_mock_time(3);
        let caching = CachingLocal::empty().until_next_block();
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(6);

        let result = caching.get_cache::<String>(key).await?;
        let expected = None;

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_set_cache_without_previous_value() -> Result<(), NovaXError> {
        let caching = CachingLocal::empty();
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
        let caching = CachingLocal::empty();
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
        let caching = CachingLocal::empty().with_duration(10);
        let key = 1;

        caching.set_cache(key, &"old value".to_string()).await?;

        set_mock_time(11);

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
}