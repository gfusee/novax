## Cloning Blockchain Data for Precision Mocking with NovaX

NovaX empowers developers with the ability to precisely replicate blockchain data, allowing you to craft an almost real-world mocking environment. This process encompasses:

- Retrieving contract storage data.
- Fetching balances associated with specific addresses.
- Procuring contract codes (a feature set to shine when NovaX completes integration with the MultiversX Virtual Machine).

> ⚠️ **Caution:** Cloning data from multiple addresses can be bandwidth-intensive. It's a best practice to utilize your own gateway by configuring your own Observing Squad for this purpose.

### Fetching Data

Start by directing NovaX to the specific addresses you wish to clone:

```rust,ignore
# extern crate tokio;
# extern crate novax;
# extern crate novax_mocking;
#
use novax_mocking::world::infos::ScenarioWorldInfos;

#[tokio::main]
async fn main() {
    let infos = ScenarioWorldInfos::fetch(
        "https://gateway.multiversx.com",
        &vec![
            "erd1qqqqqqqqqqqqqpgqq67uv84ma3cekpa55l4l68ajzhq8qm3u0n4s20ecvx".into(),
            "erd1qqqqqqqqqqqqqpgquu5rsa4ee6l4azz6vdu4hjp8z4p6tt8m0n4suht3dy".into(),
            "erd1932eft30w753xyvme8d49qejgkjc09n5e49w4mwdjtm0neld797su0dlxp".into(),
            // ... add more addresses as needed
        ]
    ).await;
}
```

### Constructing the World

Using the `ScenarioWorldInfos`, generate a `ScenarioWorld`. Remember, you must still define contract codes (unless NovaX integrates the MultiversX Virtual Machine):

```rust,ignore
# extern crate tokio;
# extern crate novax;
# extern crate novax_mocking;
#
# use std::sync::Arc;
# use tokio::sync::Mutex;
# use novax::Address;
# use novax::executor::StandardMockExecutor;
# use novax_mocking::ScenarioWorld;
#
use novax_mocking::world::infos::ScenarioWorldInfos;

#[tokio::main]
async fn main() {
    let infos = ScenarioWorldInfos::fetch(
        "https://gateway.multiversx.com",
        &vec![
            "erd1qqqqqqqqqqqqqpgqq67uv84ma3cekpa55l4l68ajzhq8qm3u0n4s20ecvx".into(),
            "erd1qqqqqqqqqqqqqpgquu5rsa4ee6l4azz6vdu4hjp8z4p6tt8m0n4suht3dy".into(),
            "erd1932eft30w753xyvme8d49qejgkjc09n5e49w4mwdjtm0neld797su0dlxp".into(),
            // ... add more addresses as needed
        ]
    ).await;

    let world = infos.into_world(|address, code_expr, world| {
        let wegld_usdc_pair: Address = "erd1qqqqqqqqqqqqqpgqq67uv84ma3cekpa55l4l68ajzhq8qm3u0n4s20ecvx".into();
        let wegld_mex_pair: Address = "erd1qqqqqqqqqqqqqpgquu5rsa4ee6l4azz6vdu4hjp8z4p6tt8m0n4suht3dy".into();

        if address == wegld_usdc_pair.to_bytes() || address == wegld_mex_pair.to_bytes() {
            // register the contract here, it looks like the below comment:
            // world.register_contract(code_expr, pair::ContractBuilder)
        }
    });

    let executor = StandardMockExecutor::new(Arc::new(Mutex::new(world)), None);
    // You're now set up to perform mocked queries, calls, and deploys
}
```

### (Optional) Saving Data to a File

Given that cloning can be network-intensive and blockchain states are ever-evolving, it's beneficial to store your cloned data into a file:

```rust,ignore
# extern crate tokio;
# extern crate novax;
# extern crate novax_mocking;
#
# use novax::Address;
#
use novax_mocking::world::infos::ScenarioWorldInfos;

#[tokio::main]
async fn main() {
    let infos = ScenarioWorldInfos::fetch(
        "https://gateway.multiversx.com",
        &vec![
            "erd1qqqqqqqqqqqqqpgqq67uv84ma3cekpa55l4l68ajzhq8qm3u0n4s20ecvx".into(),
            "erd1qqqqqqqqqqqqqpgquu5rsa4ee6l4azz6vdu4hjp8z4p6tt8m0n4suht3dy".into(),
            "erd1932eft30w753xyvme8d49qejgkjc09n5e49w4mwdjtm0neld797su0dlxp".into(),
            // ... add more addresses as needed
        ]
    ).await;

    infos.save_into_file("clone.json");
}
```

Later, you can load this data as needed:

```rust,ignore
# extern crate tokio;
# extern crate novax;
# extern crate novax_mocking;
#
#
use novax_mocking::world::infos::ScenarioWorldInfos;

#[tokio::main]
async fn main() {
let infos = ScenarioWorldInfos::from_file("clone.json").unwrap();
// Proceed with other operations
}
```

That's it! With this sophisticated technique, you're poised to create exceptionally accurate backend tests.