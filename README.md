# 🌌 NovaX: The Ultimate MultiversX Blockchain Toolkit

NovaX is your go-to toolkit for building robust software seamlessly interacting with the MultiversX blockchain. Harness the power of smart contract interactions, code generation, and automatic type conversion all checked at compile time to ensure maximum safety and efficiency.

[![Crates.io](https://img.shields.io/crates/v/novax.svg)](https://crates.io/crates/novax)
[![Documentation](https://docs.rs/novax/badge.svg)](https://docs.rs/novax)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

<div style="text-align: center;"><img src="https://github.com/gfusee/novax/blob/main/readme-demo.gif?raw=true"></div>
<div style="text-align: center;">Estimating a WEGLD -> USDC swap result </div>

## 🔥 Features

- **Code Generation:** Effortlessly generate client code from ABIs. Enjoy autocompletion for all the endpoints and views of the smart contract in the ABI, simplifying your development process.
- **Safe Contract Interactions:** Type-safe call and query functionalities ensure everything is checked at compile time.
- **Error handling:** All errors are handled through the `Result` type.
- **Native Rust Type Conversion:** Automatic conversion of complex types like `ManagedBuffer`, `ManagedVec`, etc., into native Rust types such as `String`, `Vec`, and more, whether for input or output.
- **Auto-caching:** Specify your caching strategy (local, Redis, etc.), and let NovaX handle the rest. Compose multiple strategies and opt to lock operations on similar interactions to minimize redundant requests.
- **Blockchain Information Retrieval:** Fetch token properties and address balances with ease.
- **Mocked Blockchain Interactions:** Execute requests in a mocked environment with just a single line of code.
- **Blockchain Cloning:** Need to mock an environment mirroring the current blockchain state? Specify the addresses to clone, and NovaX will fetch the storage, code, and balances of those addresses. Save and load cloned data to a file for consistent environments across executions—ideal for integration testing.

## 💡 Use Cases

NovaX is versatile and tailored for developers looking to build robust, efficient, and testable solutions on top of the MultiversX blockchain. Here are some scenarios where NovaX shines:

### 🖥️ Efficient, Safe, and Testable Backends
Build backend systems with confidence. NovaX's type-safety and autocompletion enable you to interact with the blockchain with less worry about runtime errors. Its mocking and cloning features allow for thorough testing to ensure your backend remains robust.

### 🤖 Creating Bots
Developing bots to interact with the blockchain has never been easier. Whether you are building trading bots, monitoring bots, or any other automated system, NovaX provides the tools to ensure your bot operates reliably and efficiently.

### 🚀 Deployment and Interaction Scripts
Craft scripts for deploying contracts, managing upgrades, or interacting with existing contracts on the blockchain. NovaX’s code generation from ABIs simplifies script creation and execution, saving time and reducing errors.

### 📊 Generating Data Reports
Generate insightful data reports from blockchain data. Easily fetch, analyze, and report on token properties, address balances, and contract interactions. NovaX’s caching feature minimizes the load on the blockchain, ensuring your data reporting is efficient and timely.

### 🎭 Mock and Simulate Transactions Easily
Mocking and simulation are essential for testing and verifying your smart contract transactions. With NovaX, easily create mocked environments and simulate transactions to ensure your smart contract logic is flawless before deploying to the real blockchain.

## 🛠 Installation

To truly understand NovaX, explore the [comprehensive cookbook](https://gfusee.github.io/novax/). If you haven't checked it out yet, we highly recommend doing so!

Follow these steps to seamlessly integrate NovaX into your project:

### 1️⃣ Create Directories
Create a folder named `.novax` at the root of your project. Inside `.novax`, create a subfolder called `abis` where you'll store all the ABIs of the contracts you wish to interact with.

```bash
mkdir -p .novax/abis
```

### 2️⃣ Set Environment Variable
Inform Cargo and the library about the location of the `abis` folder by setting an environment variable. Ideally, add the following snippet to your `.cargo/config.toml`:

```toml
[env]
NOVAX_PATH = { value = ".novax", relative = true }
```

### 3️⃣ Update Dependencies
Add `novax` to your `Cargo.toml` file under the `[dependencies]` section:

```toml
[dependencies]
novax = "0.0.22"
```

### 4️⃣ Build and Enjoy!
Now, build your project and dive into the development. For a richer experience, enable autocompletion by ensuring your IDE re-indexes the Cargo project:
- In VSCode, the re-indexing typically happens automatically.
- In JetBrains based IDEs such as CLion, navigate to the Cargo panel on the right, and click on the refresh icon.

```bash
cargo build
```

With these steps completed, you're all set to make the most out of the NovaX toolkit!

## 🛠 Documentation and examples

A [comprehensive cookbook](https://gfusee.github.io/novax/) is available to help you using NovaX.

You can also check out the following examples:

- [Adder](https://github.com/gfusee/novax-adder-example) - A dead simple example that show you how to deploy, query and call a contract.
- [xExchange Price Getter](https://github.com/gfusee/novax-price-getter) - This repository offers a comprehensive guide on deriving the price of tokens listed on xExchange. Notably, it demonstrates how to effectively conduct integration tests using the _novax-mocking_ crate, providing both a practical and educational perspective.
