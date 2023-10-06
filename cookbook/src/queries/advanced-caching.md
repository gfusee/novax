## Advanced Caching Strategies

While the foundational caching strategies provided by NovaX are suitable for most applications, there are times when developers may face unique challenges that require a bit more sophistication. This chapter dives into some of the more intricate caching techniques available in the `novax-caching` crate.

### CachingLocked

Basic caching strategies aim to prevent repeated requests for the same data. But how do they behave under concurrent scenarios, especially when multiple tasks are trying to fetch the same data simultaneously?

Consider the following example:

```rust,ignore
# extern crate tokio;
# extern crate novax;
# extern crate novax_caching;
#
# use novax::pair::pair::PairContract;
# use crate::novax::caching::CachingStrategy;
# use novax_caching::local::caching_local::CachingLocal;
#
use tokio::join;

#[tokio::main]
async fn main() {
let caching = CachingLocal::empty();

    let (first_result, second_result) = join!(
        fetch_pair_first_token_id(caching.with_duration(60)),
        fetch_pair_first_token_id(caching.with_duration(60))
    );
    
    println!("{}, {}", first_result, second_result);
}

async fn fetch_pair_first_token_id<C: CachingStrategy>(
caching: C
) -> String {
let pair_contract = PairContract::new("erd1qqqqqqqqqqqqqpgqeel2kumf0r8ffyhth7pqdujjat9nx0862jpsg2pqaq");

    pair_contract
        .query("https://gateway.multiversx.com")
        .with_caching_strategy(&caching)
        .get_first_token_id()
        .await
        .unwrap()
}
```

Now, you might anticipate that because we're using caching, one of the `fetch_pair_first_token_id` calls will populate the cache, and the other will then retrieve the data from it. However, here lies the nuance: the `join!` macro triggers both asynchronous functions to run concurrently. This means that `second_result` might begin its request even before the `first_result` has had a chance to populate the cache.

So, in this concurrent scenario, we're not really benefiting from our caching mechanism because both tasks might end up making separate requests to fetch the data before it's stored in the cache.

To enhance concurrency and push your code's efficiency to the extreme, employ the `CachingLocked` wrapper around your caching strategy. Here's how:

```rust,ignore
# extern crate tokio;
# extern crate novax;
# extern crate novax_caching;
#
# use novax::pair::pair::PairContract;
# use crate::novax::caching::CachingStrategy;
# use novax_caching::local::caching_local::CachingLocal;
#
use tokio::join;
use novax_caching::locked::caching::CachingLocked;

#[tokio::main]
async fn main() {
    let caching = CachingLocked::new(CachingLocal::empty());

    let (first_result, second_result) = join!(
        fetch_pair_first_token_id(caching.with_duration(60)),
        fetch_pair_first_token_id(caching.with_duration(60))
    );

    println!("{}, {}", first_result, second_result);
}

async fn fetch_pair_first_token_id<C: CachingStrategy>(
    caching: C
) -> String {
    // no change to the implementation
    # "".to_string()
}
```

By making this slight adjustment, one of the two concurrent requests is guaranteed to retrieve data from the cache.

> **Note 1:** You can wrap any struct that implements the `CachingStrategy` trait with `CachingLocked`.
> **Note 2:** Cloning a `CachingLocked` effectively clones the underlying `CachingStrategy`.

### CachingMulti

Suppose you aim to utilize multiple caching mechanisms simultaneously, such as In-Memory and Redis caching. `CachingMulti` is your go-to utility. This wrapper allows seamless integration of multiple caching strategies.

In the following example, we hypothesize that the `novax-caching` crate offers a `CachingRedis` struct. As of this writing, this feature is not available, but it's on the horizon.

```rust,ignore
# extern crate tokio;
# extern crate novax;
# extern crate novax_caching;
# extern crate async_trait;
# extern crate serde;
#
# use crate::novax::caching::CachingStrategy;
# use novax_caching::local::caching_local::CachingLocal;
# use async_trait::async_trait;
# use novax::errors::NovaXError;
# use std::future::Future;
# use serde::Serialize;
# use serde::Deserialize;
#
# #[derive(Serialize, Deserialize, Clone, Debug)]
# struct CachingRedis;
#
# impl CachingRedis {
#     pub fn new() -> Self {
#         CachingRedis
#     }
# }
#
# #[async_trait]
# impl CachingStrategy for CachingRedis {
#     async fn get_cache<T: serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(&self, key: u64) -> Result<Option<T>, NovaXError> {
#         todo!()
#     }
#
#     async fn set_cache<T: serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(&self, key: u64, value: &T) -> Result<(), NovaXError> {
#         todo!()
#     }
#
#     async fn get_or_set_cache<T, FutureGetter, Error>(&self, key: u64, getter: FutureGetter) -> Result<T, Error> where T: serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync, FutureGetter: Future<Output=Result<T, Error>> + Send, Error: From<NovaXError> {
#         todo!()
#     }
#
#     fn with_duration(&self, duration: u64) -> Self {
#         todo!()
#     }
#
#     fn until_next_block(&self) -> Self {
#         todo!()
#     }
# }
#
use novax_caching::multi::caching::CachingMulti;

#[tokio::main]
async fn main() {
    let in_memory = CachingLocal::empty();
    let redis = CachingRedis::new();
    let caching = CachingMulti::new(in_memory, redis);

    // Proceed with your caching operations
}
```

The sequence in which caching mechanisms are wrapped in `CachingMulti` is crucial:
- If the first caching strategy already holds the requested data, the subsequent ones are bypassed.
- When data from a request is returned, it's stored across all specified caching strategies.

> **Note 1:** Cloning a `CachingMulti` results in cloning all its underlying caching strategies.

> **Note 2:** The versatility of `CachingMulti` lies in its ability to encapsulate any type implementing the `CachingStrategy` trait. Given that `CachingMulti` itself adheres to the `CachingStrategy` trait, it permits nesting—meaning one can encapsulate multiple `CachingMulti` instances, thus integrating three, four, or even more caching strategies.

> **Warning:** As data is simultaneously set across all caching strategies, the efficiency of `CachingMulti` corresponds to the least efficient among its underlying strategies.
