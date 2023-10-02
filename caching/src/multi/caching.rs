use std::future::Future;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::join;
use novax::caching::CachingStrategy;
use novax::errors::NovaXError;

#[derive(Clone, Debug)]
pub struct CachingMulti<C1, C2>
where
    C1: CachingStrategy,
    C2: CachingStrategy
{
    pub first: C1,
    pub second: C2
}

impl<C1, C2> CachingMulti<C1, C2>
    where
        C1: CachingStrategy,
        C2: CachingStrategy
{
    pub fn new(first_caching: C1, second_caching: C2) -> Self {
        CachingMulti {
            first: first_caching,
            second: second_caching
        }
    }
}

#[async_trait]
impl<C1, C2> CachingStrategy for CachingMulti<C1, C2>
where
    C1: CachingStrategy,
    C2: CachingStrategy
{
    async fn get_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64) -> Result<Option<T>, NovaXError> {
        let first_cached_value = self.first.get_cache(key).await?;

        if let Some(value) = first_cached_value {
            return Ok(value)
        }

        self.second.get_cache(key).await
    }

    async fn set_cache<T: Serialize + DeserializeOwned + Send + Sync>(&self, key: u64, value: &T) -> Result<(), NovaXError> {
        let results = join!(
            self.first.set_cache(key, value),
            self.second.set_cache(key, value)
        );

        results.0?;
        results.1?;

        Ok(())
    }

    async fn get_or_set_cache<T, FutureGetter, Error>(&self, key: u64, getter: FutureGetter) -> Result<T, Error>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        FutureGetter: Future<Output=Result<T, Error>> + Send,
        Error: From<NovaXError>
    {
        if let Some(cached_value) = self.get_cache(key).await? {
            Ok(cached_value)
        } else {
            let value = getter.await?;
            self.set_cache(key, &value).await?;
            Ok(value)
        }
    }

    fn with_duration(&self, duration: u64) -> Self {
        CachingMulti::new(
            self.first.with_duration(duration),
            self.second.with_duration(duration)
        )
    }

    fn until_next_block(&self) -> Self {
        CachingMulti::new(
            self.first.until_next_block(),
            self.second.until_next_block()
        )
    }
}

#[cfg(test)]
mod test {
    use novax::caching::CachingStrategy;
    use novax::errors::NovaXError;
    use crate::local::caching_local::CachingLocal;
    use crate::multi::caching::CachingMulti;
    use crate::date::get_current_timestamp::set_mock_time;

    #[tokio::test]
    async fn test_get_cache_no_cache() -> Result<(), NovaXError> {
        let key = 1u64;

        let first_caching = CachingLocal::empty();

        let second_caching = CachingLocal::empty();

        let caching = CachingMulti::new(first_caching, second_caching);

        let result = caching.get_cache::<String>(key).await?;
        let expected: Option<String> = None;

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_first_only() -> Result<(), NovaXError> {
        let key = 1u64;
        let value = "test".to_string();
        let first_caching = CachingLocal::empty();
        first_caching.set_cache(key, &value).await?;

        let second_caching = CachingLocal::empty();

        let caching = CachingMulti::new(first_caching, second_caching);

        let result = caching.get_cache::<String>(key).await?;
        let expected: Option<String> = Some("test".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_second_only() -> Result<(), NovaXError> {
        let key = 1u64;
        let value = "test".to_string();
        let first_caching = CachingLocal::empty();

        let second_caching = CachingLocal::empty();
        second_caching.set_cache(key, &value).await?;

        let caching = CachingMulti::new(first_caching, second_caching);

        let result = caching.get_cache::<String>(key).await?;
        let expected: Option<String> = Some("test".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_first_and_second() -> Result<(), NovaXError> {
        let key = 1u64;
        let first_value = "test1".to_string();
        let first_caching = CachingLocal::empty();
        first_caching.set_cache(key, &first_value).await?;

        let second_value = "test2".to_string();
        let second_caching = CachingLocal::empty();
        second_caching.set_cache(key, &second_value).await?;

        let caching = CachingMulti::new(first_caching, second_caching);

        let result = caching.get_cache::<String>(key).await?;
        let expected: Option<String> = Some("test1".to_string());

        assert_eq!(result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_set_cache() -> Result<(), NovaXError> {
        let key = 1u64;
        let value = "test".to_string();

        let first_caching = CachingLocal::empty();
        let second_caching = CachingLocal::empty();

        let caching = CachingMulti::new(first_caching.clone(), second_caching.clone());
        caching.set_cache(key, &value).await?;

        let first_result = first_caching.get_cache::<String>(key).await?;
        let second_result = second_caching.get_cache::<String>(key).await?;
        let expected: Option<String> = Some("test".to_string());

        assert_eq!(first_result, expected);
        assert_eq!(second_result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_with_duration_before_expiration() -> Result<(), NovaXError> {
        let key = 1u64;
        let value = "test".to_string();

        let first_caching = CachingLocal::empty();
        let second_caching = CachingLocal::empty();

        let caching = CachingMulti::new(
            first_caching.clone(),
            second_caching.clone()
        )
            .with_duration(10);
        caching.set_cache(key, &value).await?;

        set_mock_time(10);

        let first_result = first_caching.get_cache::<String>(key).await?;
        let second_result = second_caching.get_cache::<String>(key).await?;
        let expected: Option<String> = Some("test".to_string());

        assert_eq!(first_result, expected);
        assert_eq!(second_result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_with_duration_after_expiration() -> Result<(), NovaXError> {
        let key = 1u64;
        let value = "test".to_string();

        let first_caching = CachingLocal::empty();
        let second_caching = CachingLocal::empty();

        let caching = CachingMulti::new(
            first_caching.clone(),
            second_caching.clone()
        )
            .with_duration(10);
        caching.set_cache(key, &value).await?;

        set_mock_time(11);

        let first_result = first_caching.get_cache::<String>(key).await?;
        let second_result = second_caching.get_cache::<String>(key).await?;
        let expected: Option<String> = None;

        assert_eq!(first_result, expected);
        assert_eq!(second_result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_until_next_block_current_block() -> Result<(), NovaXError> {
        set_mock_time(3);
        let key = 1u64;
        let value = "test".to_string();

        let first_caching = CachingLocal::empty();
        let second_caching = CachingLocal::empty();

        let caching = CachingMulti::new(
            first_caching.clone(),
            second_caching.clone()
        )
            .until_next_block();
        caching.set_cache(key, &value).await?;

        set_mock_time(5);

        let first_result = first_caching.get_cache::<String>(key).await?;
        let second_result = second_caching.get_cache::<String>(key).await?;
        let expected: Option<String> = Some("test".to_string());

        assert_eq!(first_result, expected);
        assert_eq!(second_result, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_until_next_block_next_block() -> Result<(), NovaXError> {
        set_mock_time(3);
        let key = 1u64;
        let value = "test".to_string();

        let first_caching = CachingLocal::empty();
        let second_caching = CachingLocal::empty();

        let caching = CachingMulti::new(
            first_caching.clone(),
            second_caching.clone()
        )
            .until_next_block();
        caching.set_cache(key, &value).await?;

        set_mock_time(6);

        let first_result = first_caching.get_cache::<String>(key).await?;
        let second_result = second_caching.get_cache::<String>(key).await?;
        let expected: Option<String> = None;

        assert_eq!(first_result, expected);
        assert_eq!(second_result, expected);

        Ok(())
    }
}