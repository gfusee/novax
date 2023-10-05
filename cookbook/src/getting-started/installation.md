## Setting Up NovaX: A Simple 4-Step Process

Integrating NovaX into your Rust environment is straightforward. Just follow this four-step process:

1. **Adding the NovaX Crate to Your Project**:
   Start by including the "novax" crate in your `Cargo.toml`. Ensure you're referencing the correct version (0.0.1 for this guide).

   ```toml
   [dependencies]
   novax = "0.0.1"
   ```

2. **Setting Up Your ABIs**:
   For NovaX to function properly, you need to organize your ABIs in a specific structure. Here's a simple representation of how your directory should look:

   ```text
   .
   ├── .novax
   │   └── abis
   │       ├── my_first_abi.abi.json
   │       └── my_second_abi.abi.json
   ├── src
   ├── Cargo.toml
   └── ... other project files and directories ...
   ```

   Create a folder named `.novax` at the root of your project. Inside it, establish a mandatory sub-directory named "abis" where you'll place all your ABI files, like `my_first_abi.json` and `my_second_abi.json`.

3. **Directing NovaX to Your ABIs**:
   To help NovaX locate your `.novax` directory, set the `NOVAX_PATH` environment variable. The recommended way is by adjusting the Cargo configuration. Navigate to `.config/config.toml` at the root of your project and input:

   ```toml
   [env]
   NOVAX_PATH={ value = ".novax", relative = true }
   ```

4. **Test Your Setup**:
   Now that you've configured everything, let's ensure the setup is working correctly. Build your project by running the following command:

   ```shell
   cargo build
   ```

   If everything is set up correctly, the command should run without any errors.