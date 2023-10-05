## Efficient Caching with NovaX

NovaX comes equipped with built-in capabilities for efficient caching. In this chapter, we'll explore the different caching features NovaX provides.

Before diving in, ensure you've added the `novax-caching` crate to your `Cargo.toml`:

```toml
novax-caching = "0.0.1"
```

**Note**: `novax-caching` is an extension of NovaX that provides common and tested caching strategies.

### Caching With Duration

`novax-caching` provides a struct `CachingLocal` that represents in-memory caching. This is the most basic type of caching where all data is stored in memory. It offers speed but does not persist after the program is shut down.

Let's integrate in-memory caching into the previous example, where we fetched the first token identifier of the xExchange's Pair contract:

```rust
# extern crate tokio;
# extern crate novax;
# extern crate novax_caching;

# use novax::pair::pair::PairContract;
use crate::novax::caching::CachingStrategy;
use novax_caching::local::caching_local::CachingLocal;

#[tokio::main]
async fn main() {
    let caching = CachingLocal::empty();
    let pair_contract = PairContract::new("erd1qqqqqqqqqqqqqpgqeel2kumf0r8ffyhth7pqdujjat9nx0862jpsg2pqaq");

    let first_result = pair_contract.clone()
        .query("https://gateway.multiversx.com")
        .with_caching_strategy(&caching.with_duration(60 * 60 * 24))
        .get_first_token_id()
        .await
        .expect("Failed to fetch the token ID");

    let second_result = pair_contract
        .query("https://gateway.multiversx.com")
        .with_caching_strategy(&caching.with_duration(60 * 60 * 24))
        .get_first_token_id()
        .await
        .expect("Failed to fetch the token ID");

    println!("{}, {}", first_result, second_result);
}

```

The `with_caching_strategy` method indicates: "If the result exists in the provided cache, use it. Otherwise, fetch the data from the blockchain and store it in the cache." In this example, the token identifier is fetched only for `first_result`. The `second_result` retrieves data from the cache, saving you an additional request!

**Important**: `CachingLocal` utilizes a `HashMap` wrapped in an `Arc`. When you clone a `CachingLocal`, the cloned version still modifies the same `HashMap`. Thus, the following code, which uses a cloned cache, behaves identically to the one above:

```rust
# extern crate tokio;
# extern crate novax;
# extern crate novax_caching;

# use novax::pair::pair::PairContract;
use crate::novax::caching::CachingStrategy;
use novax_caching::local::caching_local::CachingLocal;

#[tokio::main]
async fn main() {
    let caching = CachingLocal::empty();
    let cloned_caching = caching.clone();
    
    let pair_contract = PairContract::new("erd1qqqqqqqqqqqqqpgqeel2kumf0r8ffyhth7pqdujjat9nx0862jpsg2pqaq");

    let first_result = pair_contract.clone()
        .query("https://gateway.multiversx.com")
        .with_caching_strategy(&cloned_caching.with_duration(60 * 60 * 24))
        .get_first_token_id()
        .await
        .expect("Failed to fetch the token ID");

    let second_result = pair_contract
        .query("https://gateway.multiversx.com")
        .with_caching_strategy(&caching.with_duration(60 * 60 * 24))
        .get_first_token_id()
        .await
        .expect("Failed to fetch the token ID");

    println!("{}, {}", first_result, second_result);
}
```

> **Note on Versatility**: NovaX is designed with intelligence at its core. You can utilize a single caching variable for all your app's queries, regardless of their differences. NovaX discerns whether two queries are identical by assessing the contract address, the view/endpoint name, and the arguments provided. This means less manual cache management on your end and more efficient querying out of the box.

### Cache Until the Next Block

In the blockchain world, executing the same query within a single block will invariably yield the same result. Recognizing this invariant, NovaX provides a caching mechanism that retains the result only until the current block concludes. This strategy is particularly effective for queries whose outcomes might differ with every new block, such as swap estimations.

```rust
# extern crate tokio;
# extern crate num_bigint;
# extern crate novax;
# extern crate novax_caching;

# use novax::pair::pair::PairContract;
# use num_bigint::BigUint;
use crate::novax::caching::CachingStrategy;
use novax_caching::local::caching_local::CachingLocal;

#[tokio::main]
async fn main() {
let caching = CachingLocal::empty();

    let pair_contract = PairContract::new("erd1qqqqqqqqqqqqqpgqeel2kumf0r8ffyhth7pqdujjat9nx0862jpsg2pqaq");

    let result = pair_contract
        .query("https://gateway.multiversx.com")
        .with_caching_strategy(&caching.until_next_block())
        .get_amount_out(
            &"WEGLD-bd4d79".to_string(),
            &BigUint::from(10u8).pow(18)
        )
        .await
        .expect("Failed to fetch the swap estimate.");

    println!("{}", result);
}
```

After familiarizing yourself with this chapter, you should be well-equipped to code highly efficient programs. But if you're seeking even more proficiency, stay tuned: the next chapter delves into advanced caching techniques.