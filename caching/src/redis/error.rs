use novax::errors::CachingError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum CachingRedisError {
    CannotOpenConnection { description: String },
    CannotGetConnection { description: String },
    CannotGetValue { description: String },
    CannotSetValue { description: String },
    CannotClearAllValues { description: String }
}

impl CachingRedisError {
    pub fn get_description(&self) -> String {
        match self {
            CachingRedisError::CannotOpenConnection { description } => {
                format!("Cannot open a connection to the redis server: {description}")
            }
            CachingRedisError::CannotGetConnection { description } => {
                format!("Cannot get the connection to the redis server: {description}")
            }
            CachingRedisError::CannotGetValue { description } => {
                format!("Cannot get the value from the redis server: {description}")
            }
            CachingRedisError::CannotSetValue { description } => {
                format!("Cannot set the value to the redis server: {description}")
            }
            CachingRedisError::CannotClearAllValues { description } => {
                format!("Cannot clear all the values in the redis server: {description}")
            }
        }
    }
    pub fn get_type(&self) -> String {
        "CachingRedisError".to_string()
    }

    pub fn get_code(&self) -> usize {
        match self {
            CachingRedisError::CannotOpenConnection { .. } => 0,
            CachingRedisError::CannotGetConnection { .. } => 1,
            CachingRedisError::CannotGetValue { .. } => 2,
            CachingRedisError::CannotSetValue { .. } => 3,
            CachingRedisError::CannotClearAllValues { .. } => 4
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