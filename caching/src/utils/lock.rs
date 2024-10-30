use async_trait::async_trait;
use std::fmt::Debug;
use tokio::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[async_trait]
pub trait MutexLike: Send + Sync + Debug {
    type T: Send;
    fn new(value: Self::T) -> Self;

    async fn lock(&self) -> MutexGuard<'_, Self::T>;
}

#[async_trait]
impl<T: Send + Debug> MutexLike for Mutex<T> {
    type T = T;

    fn new(value: Self::T) -> Self {
        Self::new(value)
    }

    async fn lock(&self) -> MutexGuard<'_, Self::T> {
        Mutex::lock(self).await
    }
}

#[async_trait]
pub trait Locker: Send + Sync {
    type T;
    fn new(value: Self::T) -> Self;
    async fn read(&self) -> RwLockReadGuard<'_, Self::T>;
    async fn write(&self) -> RwLockWriteGuard<'_, Self::T>;
}

#[async_trait]
impl<T: Send + Sync> Locker for RwLock<T> {
    type T = T;

    fn new(value: T) -> Self {
        Self::new(value)
    }

    async fn read(&self) -> RwLockReadGuard<'_, T> {
        RwLock::read(self).await
    }

    async fn write(&self) -> RwLockWriteGuard<'_, T> {
        RwLock::write(self).await
    }
}