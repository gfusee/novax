use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use novax::caching::{CachingDurationStrategy, CachingStrategy};
use novax::errors::NovaXError;

pub type CachingLocked<C: CachingStrategy> = BaseCachingLocked<C, Arc<RwLock<()>>>;

#[async_trait]
pub trait Locker: Send + Sync + Clone + Debug {
    fn new() -> Self;
    async fn read(&self) -> RwLockReadGuard<'_, ()>;
    async fn write(&self) -> RwLockWriteGuard<'_, ()>;
}

#[async_trait]
impl Locker for Arc<RwLock<()>> {
    fn new() -> Self {
        Self::new(RwLock::new(()))
    }

    async fn read(&self) -> RwLockReadGuard<'_, ()> {
        self.read().await
    }

    async fn write(&self) -> RwLockWriteGuard<'_, ()> {
        self.write().await
    }
}

#[derive(Clone, Debug)]
pub struct BaseCachingLocked<C: CachingStrategy, L: Locker> {
    pub caching: C,
    _lockers_map: Arc<Mutex<HashMap<u64, L>>>
}

impl<C: CachingStrategy, L: Locker> BaseCachingLocked<C, L> {
    pub fn new(caching: C) -> BaseCachingLocked<C, L> {
        BaseCachingLocked {
            caching,
            _lockers_map: Arc::new(Mutex::new(HashMap::new()))
        }
    }
}

impl<C: CachingStrategy, L: Locker> BaseCachingLocked<C, L> {
    async fn get_locker(&self, key: u64) -> Result<L, NovaXError> {
        let mut lockers_map = self._lockers_map.lock().await;
        let locker = if let Some(locker) = lockers_map.get(&key) {
            locker.clone()
        } else {
            let locker = L::new();
            lockers_map.insert(key, locker.clone());
            locker
        };

        Ok(locker)
    }
}

#[async_trait]
impl<C: CachingStrategy, L: Locker> CachingStrategy for BaseCachingLocked<C, L> {
    async fn get_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64) -> Result<Option<T>, NovaXError> {
        let locker = self.get_locker(key).await?;
        let lock_value = locker.read().await;

        let result = self.caching.get_cache(key).await;

        drop(lock_value);
        result
    }

    async fn set_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64, value: &T) -> Result<(), NovaXError> {
        let locker = self.get_locker(key).await?;
        let lock_value = locker.write().await;

        let result = self.caching.set_cache(key, value).await;

        drop(lock_value);

        result
    }

    async fn get_or_set_cache<T, FutureGetter, Error>(&self, key: u64, getter: FutureGetter) -> Result<T, Error>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        FutureGetter: Future<Output=Result<T, Error>> + Send,
        Error: From<NovaXError>
    {
        let locker = self.get_locker(key).await?;
        let lock_value = locker.write().await;

        let result = self.caching.get_or_set_cache(key, getter).await;

        drop(lock_value);
        result
    }

    async fn clear(&self) -> Result<(), NovaXError> {
        self.caching.clear().await
    }

    fn with_duration_strategy(&self, strategy: CachingDurationStrategy) -> Self {
        Self::new(self.caching.with_duration_strategy(strategy))
    }
}

#[cfg(test)]
mod test {
    use std::future::Future;
    use std::sync::Arc;
    use std::time::Duration;

    use async_trait::async_trait;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use tokio::sync::Mutex;

    use novax::caching::{CachingDurationStrategy, CachingStrategy};
    use novax::errors::NovaXError;

    use crate::date::get_current_timestamp::set_mock_time;
    use crate::local::caching_local::CachingLocal;
    use crate::locked::caching::{BaseCachingLocked, CachingLocked};

    #[derive(Clone, Debug)]
    struct CachingLocalDelayedSet {
        caching: CachingLocal
    }

    impl CachingLocalDelayedSet {
        fn empty(duration_strategy: CachingDurationStrategy) -> Self {
            CachingLocalDelayedSet {
                caching: CachingLocal::empty(duration_strategy)
            }
        }
    }

    #[async_trait]
    impl CachingStrategy for CachingLocalDelayedSet {
        async fn get_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64) -> Result<Option<T>, NovaXError> {
            self.caching.get_cache(key).await
        }

        async fn set_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64, value: &T) -> Result<(), NovaXError> {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            self.caching.set_cache(key, value).await
        }

        async fn get_or_set_cache<T, FutureGetter, Error>(&self, key: u64, getter: FutureGetter) -> Result<T, Error>
        where
            T: Serialize + DeserializeOwned + Send + Sync,
            FutureGetter: Future<Output=Result<T, Error>> + Send,
            Error: From<NovaXError>
        {
            self.caching.get_or_set_cache(key, getter).await
        }

        async fn clear(&self) -> Result<(), NovaXError> {
            self.caching.clear().await
        }

        fn with_duration_strategy(&self, strategy: CachingDurationStrategy) -> Self {
            CachingLocalDelayedSet {
                caching: self.caching.with_duration_strategy(strategy)
            }
        }
    }

    #[tokio::test]
    async fn test_get_cache_key_not_found() -> Result<(), NovaXError> {
        let caching_local = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let caching = CachingLocked::new(caching_local);
        let key = 1;

        let result = caching.get_cache::<()>(key).await?;

        assert_eq!(result, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_set_cache() -> Result<(), NovaXError> {
        let caching_local = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let caching = CachingLocked::new(caching_local);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        let result = caching.get_cache::<String>(key).await?;
        let expected = Some("test".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_clear() -> Result<(), NovaXError> {
        let caching_local = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let caching = CachingLocked::new(caching_local);

        caching.set_cache(1, &"test".to_string()).await?;
        caching.set_cache(2, &"test2".to_string()).await?;
        caching.clear().await.unwrap();

        let first_result = caching.get_cache::<String>(1).await?;
        let second_result = caching.get_cache::<String>(2).await?;
        let expected = None;

        assert_eq!(first_result, expected);
        assert_eq!(second_result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_before_expiration() -> Result<(), NovaXError> {
        let caching_local = CachingLocal::empty(CachingDurationStrategy::Duration(Duration::from_secs(10)));
        let caching = CachingLocked::new(caching_local);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(9));

        let result = caching.get_cache::<String>(key).await?;
        let expected = Some("test".to_string());


        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_after_expiration() -> Result<(), NovaXError> {
        let caching_local = CachingLocal::empty(CachingDurationStrategy::Duration(Duration::from_secs(10)));
        let caching = CachingLocked::new(caching_local);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(11));

        let result = caching.get_cache::<String>(key).await?;
        let expected = None;

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_start_of_block() -> Result<(), NovaXError> {
        set_mock_time(Duration::from_secs(0));
        let caching_local = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let caching = CachingLocked::new(caching_local);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(5));

        let result = caching.get_cache::<String>(key).await?;
        let expected = Some("test".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_same_block() -> Result<(), NovaXError> {
        set_mock_time(Duration::from_secs(3));
        let caching_local = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let caching = CachingLocked::new(caching_local);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(5));

        let result = caching.get_cache::<String>(key).await?;
        let expected = Some("test".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_next_block() -> Result<(), NovaXError> {
        set_mock_time(Duration::from_secs(3));
        let caching_local = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let caching = CachingLocked::new(caching_local);
        let key = 1;
        let value = "test".to_string();

        caching.set_cache(key, &value).await?;

        set_mock_time(Duration::from_secs(6));

        let result = caching.get_cache::<String>(key).await?;
        let expected = None;

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_set_cache_without_previous_value() -> Result<(), NovaXError> {
        let caching_local = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let caching = CachingLocked::new(caching_local);
        let key = 1;

        let result = caching.get_or_set_cache(
            key,
            async {
                Ok::<_, NovaXError>("test".to_string())
            }
        ).await?;

        let expected = "test".to_string();

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_set_cache_with_previous_value() -> Result<(), NovaXError> {
        let caching_local = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let caching = CachingLocked::new(caching_local);
        let key = 1;

        caching.set_cache(key, &"old value".to_string()).await?;

        let result = caching.get_or_set_cache(
            key,
            async {
                Ok::<_, NovaXError>("test".to_string())
            }
        ).await?;

        let expected = "old value".to_string();

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_set_cache_with_previous_value_after_expiration() -> Result<(), NovaXError> {
        let caching_local = CachingLocal::empty(CachingDurationStrategy::Duration(Duration::from_secs(10)));
        let caching = CachingLocked::new(caching_local);
        let key = 1;

        caching.set_cache(key, &"old value".to_string()).await?;

        set_mock_time(Duration::from_secs(11));

        let result = caching.get_or_set_cache(
            key,
            async {
                Ok::<_, NovaXError>("test".to_string())
            }
        ).await?;

        let expected = "test".to_string();

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_locker_set_cache() {
        let key = 1u64;
        let first_value = "test1".to_string();
        let second_value = "test2".to_string();
        let caching_local_delayed = CachingLocalDelayedSet::empty(CachingDurationStrategy::EachBlock);
        caching_local_delayed.set_cache(key, &first_value).await.unwrap();
        let caching = CachingLocked::new(caching_local_delayed);

        let caching_cloned = caching.clone();
        let second_value_cloned = second_value.clone();
        let set_cache_handle = tokio::spawn(
            async move {
                caching_cloned.set_cache(key, &second_value_cloned).await
            }
        );

        let get_cache_handle = tokio::spawn(
            async move {
                caching.get_cache::<String>(key).await
            }
        );

        tokio::time::sleep(Duration::from_millis(300)).await;
        assert!(!set_cache_handle.is_finished());
        assert!(!get_cache_handle.is_finished());

        let get_cache_result = get_cache_handle.await.unwrap().unwrap().unwrap();

        let expected = second_value;

        assert_eq!(get_cache_result, expected);
    }

    #[tokio::test]
    async fn test_locker_get_or_set_cache_value_already_present() {
        let key = 1u64;
        let value = "test1".to_string();
        let caching_local = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        caching_local.set_cache(key, &value).await.unwrap();
        let caching = CachingLocked::new(caching_local);

        let caching_cloned = caching.clone();
        let get_or_set_cache_handle = tokio::spawn(
            async move {
                caching_cloned.get_or_set_cache::<String, _, NovaXError>(
                    key,
                    async {
                        panic!("should not be executed");
                    }
                ).await
            }
        );

        let get_cache_handle = tokio::spawn(
            async move {
                caching.get_cache::<String>(key).await
            }
        );

        tokio::time::sleep(Duration::from_millis(1000)).await;
        assert!(get_or_set_cache_handle.is_finished());
        assert!(get_cache_handle.is_finished());

        let get_or_set_cache_result = get_or_set_cache_handle.await.unwrap().unwrap();
        let get_cache_result = get_cache_handle.await.unwrap().unwrap().unwrap();

        let expected = value;

        assert_eq!(get_or_set_cache_result, expected);
        assert_eq!(get_cache_result, expected);
    }

    #[tokio::test]
    async fn test_locker_get_or_set_cache_no_previous_value() {
        let key = 1u64;
        let second_value = "test2".to_string();
        let caching_local = CachingLocal::empty(CachingDurationStrategy::EachBlock);
        let caching = CachingLocked::new(caching_local);

        let fake_value = Arc::new(Mutex::new(0u64));

        let caching_cloned = caching.clone();
        let fake_value_cloned = fake_value.clone();
        let second_value_cloned = second_value.clone();
        let get_or_set_cache_handle = tokio::spawn(
            async move {
                caching_cloned.get_or_set_cache(
                    key,
                    async {
                        let mut should_loop = true;
                        while should_loop {
                            should_loop = async {
                                let fake_value = fake_value_cloned.lock().await;
                                *fake_value == 0
                            }.await;

                            tokio::time::sleep(Duration::from_millis(300)).await;
                        }

                        Ok::<_, NovaXError>(second_value_cloned)
                    }
                ).await
            }
        );

        let get_cache_handle = tokio::spawn(
            async move {
                caching.get_cache::<String>(key).await
            }
        );

        tokio::time::sleep(Duration::from_millis(1000)).await;
        assert!(!get_or_set_cache_handle.is_finished());
        assert!(!get_cache_handle.is_finished());

        {
            let mut fake_value_guard = fake_value.lock().await;
            *fake_value_guard = 1;
        }

        let get_or_set_cache_result = get_or_set_cache_handle.await.unwrap().unwrap();
        let get_cache_result = get_cache_handle.await.unwrap().unwrap().unwrap();

        let expected = second_value;

        assert_eq!(get_or_set_cache_result, expected);
        assert_eq!(get_cache_result, expected);
    }
}