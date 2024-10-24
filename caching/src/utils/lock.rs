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
    fn new() -> Self;
    async fn read(&self) -> RwLockReadGuard<'_, ()>;
    async fn write(&self) -> RwLockWriteGuard<'_, ()>;
}

#[async_trait]
impl Locker for RwLock<()> {
    fn new() -> Self {
        Self::new(())
    }

    async fn read(&self) -> RwLockReadGuard<'_, ()> {
        RwLock::read(self).await
    }

    async fn write(&self) -> RwLockWriteGuard<'_, ()> {
        RwLock::write(self).await
    }
}