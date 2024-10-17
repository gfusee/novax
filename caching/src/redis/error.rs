use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum CachingRedisError {
    CannotOpenConnection,
    CannotGetConnection,
    CannotGetValue,
    CannotSetValue,
}