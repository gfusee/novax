use std::future::Future;
use async_trait::async_trait;
use redis::IntoConnectionInfo;
use serde::de::DeserializeOwned;
use serde::Serialize;
use novax::caching::{CachingDurationStrategy, CachingStrategy};
use novax::errors::{CachingError, NovaXError};
use crate::redis::client::RedisClient;
use crate::redis::error::CachingRedisError;

#[derive(Clone, Debug)]
pub struct CachingRedis<Client: RedisClient> {
    pub(crate) client: Client
}

impl<Client: RedisClient> CachingRedis<Client> {
    fn new<Info: IntoConnectionInfo>(info: Info) -> Result<Self, CachingRedisError> {
        let client = Client::open(info)?;

        Ok(
            CachingRedis {
                client
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
            .set(key, encoded)
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
        todo!()
    }

    async fn clear(&self) -> Result<(), NovaXError> {
        todo!()
    }

    fn with_duration_strategy(&self, strategy: CachingDurationStrategy) -> Self {
        todo!()
    }
}