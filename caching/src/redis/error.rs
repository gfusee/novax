use serde::{Deserialize, Serialize};
use novax::errors::CachingError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum CachingRedisError {
    CannotOpenConnection,
    CannotGetConnection,
    CannotGetValue,
    CannotSetValue,
    CannotClearAllValues
}

impl CachingRedisError {
    pub fn get_description(&self) -> String {
        match self {
            CachingRedisError::CannotOpenConnection => {
                "Cannot open a connection to the redis server.".to_string()
            }
            CachingRedisError::CannotGetConnection => {
                "Cannot get the connection to the redis server.".to_string()
            }
            CachingRedisError::CannotGetValue => {
                "Cannot get the value from the redis server.".to_string()
            }
            CachingRedisError::CannotSetValue => {
                "Cannot set the value to the redis server.".to_string()
            }
            CachingRedisError::CannotClearAllValues => {
                "Cannot clear all the values in the redis server.".to_string()
            }
        }
    }
    pub fn get_type(&self) -> String {
        "CachingRedisError".to_string()
    }

    pub fn get_code(&self) -> usize {
        match self {
            CachingRedisError::CannotOpenConnection => 0,
            CachingRedisError::CannotGetConnection => 1,
            CachingRedisError::CannotGetValue => 2,
            CachingRedisError::CannotSetValue => 3,
            CachingRedisError::CannotClearAllValues => 4
        }
    }
}

impl From<CachingRedisError> for CachingError {
    fn from(value: CachingRedisError) -> Self {
        CachingError::OtherError {
            description: value.get_description(),
            code: value.get_code(),
            type_name: value.get_type(),
        }
    }
}