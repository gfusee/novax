use std::fmt::Debug;

use async_trait::async_trait;
use redis::AsyncCommands;
use redis::{AsyncConnectionConfig, FromRedisValue, IntoConnectionInfo, SetExpiry, ToRedisArgs};

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
            Err(error) => Err(CachingRedisError::CannotOpenConnection { description: format!("{}", error) })
        }
    }

    async fn set<K: ToRedisArgs + Send + Sync, V: ToRedisArgs + Send + Sync>(&self, key: K, value: V, duration: u64) -> Result<(), CachingRedisError> {
        if duration == 0 {
            return Ok(());
        }

        let mut connection = match self.get_multiplexed_async_connection_with_config(&AsyncConnectionConfig::new()).await {
            Ok(connection) => connection,
            Err(error) => return Err(CachingRedisError::CannotGetConnection { description: format!("{}", error) })
        };

        let options = redis::SetOptions::default()
            .with_expiration(SetExpiry::EX(duration));

        match connection.set_options::<_, _, ()>(key, value, options).await {
            Ok(_) => Ok(()),
            Err(error) => Err(CachingRedisError::CannotSetValue { description: format!("{}", error) })
        }
    }

    async fn get<K: ToRedisArgs + Send + Sync, RV: FromRedisValue + Send + Sync>(&self, key: K) -> Result<Option<RV>, CachingRedisError> {
        let mut connection = match self.get_multiplexed_async_connection_with_config(&AsyncConnectionConfig::new()).await {
            Ok(connection) => connection,
            Err(error) => return Err(CachingRedisError::CannotGetConnection { description: format!("{}", error) })
        };

        match connection.get(key).await {
            Ok(value) => Ok(value),
            Err(error) => Err(CachingRedisError::CannotGetValue { description: format!("{}", error) })
        }
    }

    async fn clear(&self) -> Result<(), CachingRedisError> {
        let mut connection = match self.get_multiplexed_async_connection_with_config(&AsyncConnectionConfig::new()).await {
            Ok(connection) => connection,
            Err(error) => return Err(CachingRedisError::CannotGetConnection { description: format!("{}", error) })
        };

        match redis::cmd("FLUSHALL").exec_async(&mut connection).await {
            Ok(_) => Ok(()),
            Err(error) => Err(CachingRedisError::CannotClearAllValues { description: format!("{}", error) })
        }
    }
}