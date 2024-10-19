use std::future::Future;

use async_trait::async_trait;
pub use redis::ConnectionInfo;
pub use redis::IntoConnectionInfo;
pub use redis::RedisConnectionInfo;
pub use redis::RedisError;
use serde::de::DeserializeOwned;
use serde::Serialize;

use novax::caching::{CachingDurationStrategy, CachingStrategy};
use novax::errors::{CachingError, NovaXError};

use crate::date::get_current_timestamp::{get_current_timestamp, GetDuration};
use crate::redis::client::RedisClient;
use crate::redis::error::CachingRedisError;

pub type CachingRedis = BaseCachingRedis<redis::Client>;

#[derive(Clone, Debug)]
pub struct BaseCachingRedis<Client: RedisClient> {
    pub(crate) client: Client,
    pub duration_strategy: CachingDurationStrategy
}

impl<Client: RedisClient> BaseCachingRedis<Client> {
    pub fn new<Info: IntoConnectionInfo>(
        info: Info,
        duration_strategy: CachingDurationStrategy
    ) -> Result<Self, CachingRedisError> {
        let client = Client::open(info)?;

        Ok(
            BaseCachingRedis {
                client,
                duration_strategy
            }
        )
    }
}

#[async_trait]
impl<Client: RedisClient> CachingStrategy for BaseCachingRedis<Client> {
    async fn get_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64) -> Result<Option<T>, NovaXError> {
        let opt_value_encoded: Option<Vec<u8>> = self.client
            .get(key)
            .await
            .map_err(|e| {
                CachingError::from(e)
            })?;

        let Some(value_encoded) = opt_value_encoded else {
            return Ok(None);
        };

        let Ok(decoded) = bitcode::deserialize(&value_encoded) else {
            return Err(CachingError::UnableToDeserialize.into())
        };

        Ok(Some(decoded))
    }

    async fn set_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64, value: &T) -> Result<(), NovaXError> {
        let Ok(encoded) = bitcode::serialize(value) else {
            return Err(CachingError::UnableToSerialize.into())
        };

        self.client
            .set(key, encoded, self.duration_strategy.get_duration_from_now(&get_current_timestamp()?)?.as_secs())
            .await
            .map_err(|e| {
                CachingError::from(e).into()
            })
    }

    async fn get_or_set_cache<T, FutureGetter, Error>(&self, key: u64, getter: FutureGetter) -> Result<T, Error>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        FutureGetter: Future<Output=Result<T, Error>> + Send,
        Error: From<NovaXError>
    {
        let opt_value = self.get_cache(key).await?;

        match opt_value {
            None => {
                let value_to_set = getter.await?;
                self.set_cache(key, &value_to_set).await?;
                Ok(value_to_set)
            },
            Some(value) => {
                Ok(value)
            }
        }
    }

    async fn clear(&self) -> Result<(), NovaXError> {
        self.client
            .clear()
            .await
            .map_err(|e| {
                CachingError::from(e).into()
            })
    }

    fn with_duration_strategy(&self, strategy: CachingDurationStrategy) -> Self {
        BaseCachingRedis {
            client: self.client.clone(),
            duration_strategy: strategy,
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use async_trait::async_trait;
    use redis::{FromRedisValue, IntoConnectionInfo, ToRedisArgs};

    use novax::caching::{CachingDurationStrategy, CachingStrategy};
    use novax::errors::NovaXError;

    use crate::date::get_current_timestamp::set_mock_time;
    use crate::redis::client::RedisClient;
    use crate::redis::error::CachingRedisError;
    use crate::redis::redis::BaseCachingRedis;

    #[derive(Clone, Debug)]
    struct MockRedisClient;

    #[async_trait]
    impl RedisClient for MockRedisClient {
        fn open<Info: IntoConnectionInfo>(_info: Info) -> Result<Self, CachingRedisError> {
            Ok(MockRedisClient)
        }

        async fn get<K: ToRedisArgs + Send + Sync, RV: FromRedisValue + Send + Sync>(&self, key: K) -> Result<Option<RV>, CachingRedisError> {
            if key.to_redis_args() == 1.to_redis_args() { // Not found
                Ok(None)
            } else if key.to_redis_args() == 2.to_redis_args() { // Found
                Ok(Some(RV::from_byte_vec(&[2, 0, 1]).unwrap().into_iter().next().unwrap()))
            } else {
                Ok(None)
            }
        }

        async fn set<K: ToRedisArgs + Send + Sync, V: ToRedisArgs + Send + Sync>(&self, key: K, value: V, duration: u64) -> Result<(), CachingRedisError> {
            if value.to_redis_args() != bitcode::serialize("test").unwrap().to_redis_args() {
                panic!();
            }

            if key.to_redis_args() == 2.to_redis_args() { // set_cache_start_of_block
                if duration != 6 {
                    panic!();
                }

                Ok(())
            } else if key.to_redis_args() == 3.to_redis_args() { // set_cache_next_block
                if duration != 3 {
                    panic!();
                }

                Ok(())
            } else {
                Ok(())
            }
        }

        async fn clear(&self) -> Result<(), CachingRedisError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_get_cache_key_not_found() -> Result<(), NovaXError> {
        let caching = BaseCachingRedis::<MockRedisClient>::new("", CachingDurationStrategy::EachBlock).unwrap();
        let key = 1;

        let result = caching.get_cache::<()>(key).await?;

        assert_eq!(result, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_key_found() -> Result<(), NovaXError> {
        let caching = BaseCachingRedis::<MockRedisClient>::new("", CachingDurationStrategy::EachBlock).unwrap();
        let key = 2;

        let result = caching.get_cache::<Vec<u8>>(key).await?;

        assert_eq!(result, Some([0, 1].to_vec()));

        Ok(())
    }

    #[tokio::test]
    async fn test_set_cache() -> Result<(), NovaXError> {
        let caching = BaseCachingRedis::<MockRedisClient>::new("", CachingDurationStrategy::EachBlock).unwrap();
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_set_cache_start_of_block() -> Result<(), NovaXError> {
        set_mock_time(Duration::from_secs(0));
        let caching = BaseCachingRedis::<MockRedisClient>::new("", CachingDurationStrategy::EachBlock).unwrap();
        let key = 2;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_set_cache_next_block() -> Result<(), NovaXError> {
        set_mock_time(Duration::from_secs(3));
        let caching = BaseCachingRedis::<MockRedisClient>::new("", CachingDurationStrategy::EachBlock).unwrap();
        let key = 3;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_get_and_set_cache_key_not_found() -> Result<(), NovaXError> {
        let caching = BaseCachingRedis::<MockRedisClient>::new("", CachingDurationStrategy::EachBlock).unwrap();
        let key = 1;

        let result = caching.get_or_set_cache::<String, _, NovaXError>(key, async {
            // error if serialized

            Ok("test".to_string())
        }).await?;

        assert_eq!(result, "test");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_and_set_cache_key_found() -> Result<(), NovaXError> {
        let caching = BaseCachingRedis::<MockRedisClient>::new("", CachingDurationStrategy::EachBlock).unwrap();
        let key = 2;

        let result = caching.get_or_set_cache::<Vec<u8>, _, NovaXError>(key, async {
            // error if serialized

            Ok(panic!())
        }).await?;

        assert_eq!(result, vec![0u8, 1u8]);

        Ok(())
    }

    #[tokio::test]
    async fn test_clear() -> Result<(), NovaXError> {
        let caching = BaseCachingRedis::<MockRedisClient>::new("", CachingDurationStrategy::EachBlock).unwrap();

        caching.clear().await?;

        Ok(())
    }
}