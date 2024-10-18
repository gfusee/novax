use std::future::Future;
use async_trait::async_trait;
use redis::IntoConnectionInfo;
use serde::de::DeserializeOwned;
use serde::Serialize;
use novax::caching::{CachingDurationStrategy, CachingStrategy};
use novax::errors::{CachingError, NovaXError};
use crate::date::get_current_timestamp::{get_current_timestamp, GetDuration};
use crate::redis::client::RedisClient;
use crate::redis::error::CachingRedisError;

#[derive(Clone, Debug)]
pub struct CachingRedis<Client: RedisClient> {
    pub(crate) client: Client,
    pub duration_strategy: CachingDurationStrategy
}

impl<Client: RedisClient> CachingRedis<Client> {
    fn new<Info: IntoConnectionInfo>(
        info: Info,
        duration_strategy: CachingDurationStrategy
    ) -> Result<Self, CachingRedisError> {
        let client = Client::open(info)?;

        Ok(
            CachingRedis {
                client,
                duration_strategy
            }
        )
    }
}

#[async_trait]
impl<Client: RedisClient> CachingStrategy for CachingRedis<Client> {
    async fn get_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64) -> Result<Option<T>, NovaXError> {
        let opt_value_encoded: Option<Vec<u8>> = self.client
            .get(key)
            .await
            .map_err(|_| {
                CachingError::UnknownError
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
            .map_err(|_| {
                CachingError::UnknownError.into()
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
            .map_err(|_| {
                CachingError::UnknownError.into()
            })
    }

    fn with_duration_strategy(&self, strategy: CachingDurationStrategy) -> Self {
        CachingRedis {
            client: self.client.clone(),
            duration_strategy: strategy,
        }
    }
}