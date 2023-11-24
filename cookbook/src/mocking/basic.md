## Mocking with NovaX

NovaX equips developers with the capability to mock their environments seamlessly across queries, calls, and deployments. This feature is powered by the MultiversX's Rust Testing Framework. While this guide provides an overview of using the mocking capabilities, it won't delve into creating a `ScenarioWorld`. For that, kindly refer to the official documentation.
Before delving into the mocking process with NovaX, ensure you've added the necessary extension crate to your project. Update your `Cargo.toml` dependencies to include:

```toml
novax-mocking = "0.0.2"
```

Now, let's proceed with how to execute mocked queries, calls, and deploys using NovaX.

> **Note 1**: The Rust Testing Framework mandates that you possess the code of every contract you interact with. These contracts should align with the same version of `mx-sdk-rs` that NovaX uses. Specifically, NovaX 0.0.22 is in sync with `mx-sdk-rs` version 0.43.3.

> **Note 2**: For those looking to replicate the real blockchain's state into a `ScenarioWorld`, NovaX furnishes tools designed for this intricate process. We'll explore this technique further in the subsequent chapter.

> **Important**: On the horizon for NovaX is the integration of the MultiversX Virtual Machine. This advancement will eliminate the dependency on contract codes, streamlining the process to require only the .wasm files.

### Creating the Mocked Executor

If you've established a `ScenarioWorld`, spinning up a mocked executor is a straightforward process. Start by wrapping the `ScenarioWorld` instance within an `Arc<Mutex<>>` struct:

```rust,ignore
# extern crate novax;
# extern crate novax_mocking;
# extern crate tokio;
#
use std::sync::Arc;
use tokio::sync::Mutex;
use novax::executor::StandardMockExecutor;
use novax_mocking::ScenarioWorld;

let world = ScenarioWorld::new();
// Adjust the world to match your requirements...

let executor = StandardMockExecutor::new(
    Arc::new(Mutex::new(world)),
    None // This represents the transaction sender. If set to None, it defaults to the contract address.
);
```

### Executing Mocked Queries, Calls, and Deploys

With the mocked executor in place, you're now ready to send mocked queries, calls, and deployments. Just substitute your mocked executor in place of the regular one.

#### Mocked Query Example:
Here's how a previously introduced query would look when mocked:

```rust,ignore
# extern crate novax;
# extern crate novax_mocking;
# extern crate tokio;
#
# use novax::pair::pair::PairContract;
use std::sync::Arc;
use tokio::sync::Mutex;
use novax::executor::StandardMockExecutor;
use novax_mocking::ScenarioWorld;

#[tokio::main]
async fn main() {
    let world = ScenarioWorld::new();
    let executor = StandardMockExecutor::new(
        Arc::new(Mutex::new(world)),
        None // Represents the transaction sender. Defaults to contract address if None.
    );

    // Initializing the PairContract with its address
    let pair_contract = PairContract::new("erd1qqqqqqqqqqqqqpgqeel2kumf0r8ffyhth7pqdujjat9nx0862jpsg2pqaq");

    // Executing the query to obtain the first token's ID
    let result = pair_contract
        .query(executor)
        .get_first_token_id()
        .await
        .expect("Failed to fetch the token ID");

    println!("{}", result);
}
```

#### Mocked Call Example:
Similarly, calls can be mocked as shown below:

```rust,ignore
# extern crate novax;
# extern crate novax_mocking;
# extern crate tokio;
# extern crate num_bigint;
# use num_bigint::BigUint;
# use std::ops::Mul;
# use novax::transaction::TokenTransfer;
# use novax::pair::pair::PairContract;
use std::sync::Arc;
use tokio::sync::Mutex;
use novax::executor::StandardMockExecutor;
use novax_mocking::ScenarioWorld;

#[tokio::main]
async fn main() {
    let world = ScenarioWorld::new();
    let executor = StandardMockExecutor::new(
        Arc::new(Mutex::new(world)),
        None // Represents the transaction sender. Defaults to contract address if None.
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

#### Mocked Deploy Example:
Here's how deployment operations can be mocked:

```rust,ignore
# extern crate tokio;
# extern crate novax;
# extern crate novax_mocking;
#
# use novax::pair::pair::PairContract;
# use novax::executor::NetworkExecutor;
# use novax::CodeMetadata;
# use novax::code::DeployData;
use std::sync::Arc;
use tokio::sync::Mutex;
use novax::executor::StandardMockExecutor;
use novax_mocking::ScenarioWorld;

#[tokio::main]
async fn main() {
    let world = ScenarioWorld::new();
    let mut executor = StandardMockExecutor::new(
        Arc::new(Mutex::new(world)),
        Some("<deployer address>".into()) // The transaction sender, mandatory when deploying a contract
    );

    let deploy_data = DeployData {
        code: "<path to the .wasm file>",
        metadata: CodeMetadata::PAYABLE | CodeMetadata::UPGRADEABLE,
    };

    let result = PairContract::deploy(
        deploy_data,
        &mut executor,
        600_000_000, // gas limit
        &"WEGLD-d7c6bb".to_string(),
        &"USDC-8d4068".to_string(),
        &"<xexchange router address>".into(),
        &"<xexchange router owner address>".into(),
        &0, // total fee percent
        &0, // special fee percent
        &"<initial liquidity adder address>".into(),
        &vec![
            "<first admin address>".into(),
            "<second admin address>".into(),
        ]
    )
    .await
    .expect("Deployment failed.");
}
```

Equipped with this knowledge, you're now ready to craft well-tested backends! In the upcoming chapter, we'll dive into how NovaX can be instrumental in constructing tests that harness data from the real environment.