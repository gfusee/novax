use std::fmt::Debug;

use async_trait::async_trait;
use redis::{AsyncConnectionConfig, FromRedisValue, IntoConnectionInfo, SetExpiry, ToRedisArgs};
use redis::AsyncCommands;

use crate::redis::error::CachingRedisError;

#[async_trait]
pub trait RedisClient: Clone + Debug + Send + Sync {
    fn open<Info: IntoConnectionInfo>(info: Info) -> Result<Self, CachingRedisError>;
    async fn set<K: ToRedisArgs + Send + Sync, V: ToRedisArgs + Send + Sync>(&self, key: K, value: V, duration: u64) -> Result<(), CachingRedisError>;
    async fn get<K: ToRedisArgs + Send + Sync, RV: FromRedisValue + Send + Sync>(&self, key: K) -> Result<Option<RV>, CachingRedisError>;
    async fn clear(&self) -> Result<(), CachingRedisError>;
}

#[async_trait]
impl RedisClient for redis::Client {
    fn open<Info: IntoConnectionInfo>(info: Info) -> Result<Self, CachingRedisError> {
        match Self::open(info) {
            Ok(client) => Ok(client),
            Err(_) => Err(CachingRedisError::CannotOpenConnection)
        }
    }

    async fn set<K: ToRedisArgs + Send + Sync, V: ToRedisArgs + Send + Sync>(&self, key: K, value: V, duration: u64) -> Result<(), CachingRedisError> {
        let Ok(mut connection) = self.get_multiplexed_async_connection_with_config(&AsyncConnectionConfig::new()).await else {
            return Err(CachingRedisError::CannotGetConnection)
        };

        let options = redis::SetOptions::default()
            .with_expiration(SetExpiry::EX(duration));

        if !connection.set_options::<_, _, ()>(key, value, options).await.is_ok() {
            return Err(CachingRedisError::CannotSetValue)
        }

        return Ok(())
    }

    async fn get<K: ToRedisArgs + Send + Sync, RV: FromRedisValue + Send + Sync>(&self, key: K) -> Result<Option<RV>, CachingRedisError> {
        let Ok(mut connection) = self.get_multiplexed_async_connection_with_config(&AsyncConnectionConfig::new()).await else {
            return Err(CachingRedisError::CannotGetConnection)
        };

        let Ok(value) = connection.get(key).await else {
            return Err(CachingRedisError::CannotGetValue)
        };

        Ok(value)
    }

    async fn clear(&self) -> Result<(), CachingRedisError> {
        let Ok(mut connection) = self.get_multiplexed_async_connection_with_config(&AsyncConnectionConfig::new()).await else {
            return Err(CachingRedisError::CannotGetConnection)
        };

        if !redis::cmd("FLUSHALL").exec_async(&mut connection).await.is_ok() {
            return Err(CachingRedisError::CannotSetValue)
        };

        return Ok(());
    }
}