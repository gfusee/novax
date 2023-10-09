## Deploys

Deploying contracts using NovaX is a straightforward process, bearing similarities to contract calls. NovaX auto-generates a "deploy" method from the ABI. This method not only takes in the required inputs but also deployment details such as the WASM binary and metadata attributes (such as readability and upgradeability).

### Creating A Wallet

The procedure here mirrors the one in the [Basic Calls](../calls/basic.md) chapter:

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

### Performing The Deployment

Carrying out the deployment simply entails invoking the `deploy` method with the necessary parameters:

```rust,ignore
# extern crate tokio;
# extern crate novax;
#
# use novax::pair::pair::PairContract;
use novax::executor::NetworkExecutor;
use novax::Wallet;
use novax::CodeMetadata;
use novax::code::DeployData;

#[tokio::main]
async fn main() {
    let wallet = Wallet::from_pem_file("<path to your .pem file>").unwrap();
    let mut executor = NetworkExecutor::new(
        "https://devnet-gateway.multiversx.com",
        &wallet
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

    println!("Deployment successful! Contract's address: {}", result.0.to_bech32_string().expect("Failed to convert address to bech32 format."));
}
```

The result yielded is a tuple. The first element captures the new address, and the second contains the transaction outcome.

And with that, you've successfully deployed a contract using NovaX!
