## Calls

If you've gone through the [Basic Queries](../queries/basic.md), then diving into contract calls with NovaX should be a cinch.

**Note:** For this chapter, we'll be working on the devnet. This means the gateway and contract addresses might differ from previous chapters.

### Instantiate A Wallet

NovaX builds upon the official MultiversX's SDK. This means that only wallets compatible with the MultiversX's SDK can be used with NovaX. If you're working with a `.pem` wallet file, here's how you can bring it to life in NovaX:

```rust,ignore
# extern crate novax;
#
use novax::Wallet;
use novax::executor::NetworkExecutor;

let wallet = Wallet::from_pem_file("<path to your .pem file>").expect("Failed to load the wallet.");
let executor = NetworkExecutor::new(
    "https://devnet-gateway.multiversx.com",
    &wallet
);
```

### Calling The Contract

Fancy a token swap? We'll be calling upon the `swapTokensFixedOutput` endpoint of the xExchange's Pair contract. The coding pattern is quite akin to a query:

```rust,ignore
# extern crate tokio;
# extern crate novax;
# extern crate num_bigint;
#
# use novax::pair::pair::PairContract;
# use num_bigint::BigUint;
# use std::ops::Mul;
use novax::transaction::TokenTransfer;
use novax::executor::NetworkExecutor;
use novax::Wallet;

#[tokio::main]
async fn main() {
    let wallet = Wallet::from_pem_file("<path to your .pem file>").unwrap();
    let executor = NetworkExecutor::new(
        "https://devnet-gateway.multiversx.com",
        &wallet
    );

    let pair_contract = PairContract::new("erd1qqqqqqqqqqqqqpgqq67uv84ma3cekpa55l4l68ajzhq8qm3u0n4s20ecvx");

    let result_tokens = pair_contract
        .call(executor, 600_000_000) // gas limit, set to the maximum
        .with_esdt_transfers(&vec![
            TokenTransfer {
                identifier: "WEGLD-d7c6bb".to_string(),
                nonce: 0,
                amount: BigUint::from(10u8).pow(18) // 1 WEGLD
            }
        ])
        .swap_tokens_fixed_output(
            &"USDC-8d4068".to_string(), // token out
            &BigUint::from(10u8).pow(6).mul(24u8) // slippage, seeking at least 24 USDC
        )
        .await
        .expect("Failed to execute the swap.")
        .result
        .expect("No result from the swap.");

    println!(r#"
    Swap success! Received tokens:

    - {} of {},
    - {} of {}
    "#,
        result_tokens.0.amount,
        result_tokens.0.token_identifier,
        result_tokens.1.amount,
        result_tokens.1.token_identifier
    );
}
```

And with that, we've seamlessly executed a contract call using NovaX!
