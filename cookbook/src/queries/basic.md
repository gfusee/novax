## Queries

### Querying Without Parameters

Imagine you have the xExchange's Pair contract ABI, and you wish to fetch the identifier of the first token in the pair. Thanks to NovaX, you don't have to stress over manual implementations. Based on the provided ABI, NovaX auto-generates utility methods, one of which is `get_first_token_id`, tailored for this exact purpose.

Below is a concise example showcasing how to harness this auto-generated method:

```rust
# extern crate tokio;
# extern crate novax;

use novax::pair::pair::PairContract;

#[tokio::main]
async fn main() {
    // Initializing the PairContract with its address
    let pair_contract = PairContract::new("erd1qqqqqqqqqqqqqpgqeel2kumf0r8ffyhth7pqdujjat9nx0862jpsg2pqaq");

    // Executing the query to get the first token's ID
    let result = pair_contract
        .query("https://gateway.multiversx.com")
        .get_first_token_id()
        .await
        .expect("Failed to fetch the token ID");

    // Displaying the result
    println!("{}", result);
}
```

Upon executing the above code, your console should promptly display "WEGLD-bd4d79". Voilà! You've successfully fetched the identifier of the first token in the pair using NovaX.

> **Note**: The method `get_first_token_id` corresponds to the view `getFirstTokenId` in the contract. NovaX automatically adheres to Rust's naming conventions by converting endpoint and view names into snake_case.

### Querying With Parameters

NovaX generates parameters based on Rust's common types, relieving you from wrestling with the intricacies of various contract types. Before diving into the example, ensure you have added the `num-bigint` crate to your dependencies in `Cargo.toml`:

```toml
num-bigint = "0.4.4"
```

Suppose you want to estimate a swap result (an amount of USDC) through the "getAmountOut" endpoint, with an input of 1 WEGLD:

```rust
# extern crate tokio;
# extern crate num_bigint;
# extern crate novax;

use novax::pair::pair::PairContract;
use num_bigint::BigUint;

#[tokio::main]
async fn main() {
    let pair_contract = PairContract::new("erd1qqqqqqqqqqqqqpgqeel2kumf0r8ffyhth7pqdujjat9nx0862jpsg2pqaq");
    
    let result = pair_contract
        .query("https://gateway.multiversx.com")
        .get_amount_out(
            &"WEGLD-bd4d79".to_string(),
            &BigUint::from(10u8).pow(18)
        )
        .await
        .expect("Failed to fetch the swap estimate.");

    println!("{}", result);
}
```

> **Reminder:** The WEGLD amount is expressed in terms of 10^18, mainly because WEGLD has 18 decimals. When you run the code, expect to see an amount of USDC represented in the form of 10^6 since USDC has 6 decimals.