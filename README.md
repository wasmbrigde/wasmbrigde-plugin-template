# WasmBridge Plugin Template

This is a template for creating WebAssembly (WASM) plugins for the **WasmBridge** agent. It uses the [Extism](https://extism.org/) PDK for Rust.

## Prerequisites

1.  **Rust**: Ensure you have the Rust toolchain installed.
2.  **wasm32 target**: Add the wasm32-unknown-unknown target:
    ```bash
    rustup target add wasm32-unknown-unknown
    ```

## Project Structure

*   `src/lib.rs`: The main entry point for your plugin. It contains the logic for metadata, request handling, and cloud commands.
*   `Cargo.toml`: Project dependencies. It includes `extism-pdk` and the shared `plugin-protocol` crate.

## Creating Your Own Plugin

### 1. Define Plugin Metadata (`info` function)

The `info` function is used by the host to identify your plugin and its capabilities.

```rust
#[plugin_fn]
pub fn info() -> FnResult<Json<PluginInfo>> {
    let info = PluginInfo {
        name: "my-custom-plugin".into(),
        version: "0.1.0".into(),
        description: "What my plugin does...".into(),
        endpoints: vec![
            // Define routes that the host will proxy to this plugin
        ],
        settings: vec![
            // Define configuration fields (Text, Password, Number, Boolean)
        ],
    };
    Ok(Json(info))
}
```

### 2. Handle HTTP Requests (`handle_request`)

When a user or the UI makes a request to a path defined in your `endpoints`, the host calls `handle_request`.

```rust
#[plugin_fn]
pub fn handle_request(Json(req): Json<PluginRequest>) -> FnResult<Json<PluginResponse>> {
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/my-data") => {
            // Retrieve config values
            let api_url = config::get("api_url")?.unwrap_or_default();
            
            // Perform logic and return response
            Ok(Json(PluginResponse { 
                status: 200, 
                headers: Default::default(), 
                body: Some(b"Hello from Plugin!".to_vec()) 
            }))
        },
        _ => // Handle 404...
    }
}
```

### 3. Handle Cloud Commands (`execute_command`)

If your agent is connected to the Cloud Control Plane via **Reverse Push**, you can handle remote tasks here.

```rust
#[plugin_fn]
pub fn execute_command(
    Json(payload): Json<plugin_protocol::CloudCommandPayload>,
) -> FnResult<Json<plugin_protocol::CloudCommandResult>> {
    match payload.task.as_str() {
        "scan" => Ok(Json(CloudCommandResult::success("Scanning...")),
        _ => Ok(Json(CloudCommandResult::error("Unknown task")),
    }
}
```

### 4. Direct Execution Exports

In addition to the standard handlers, you can export any custom function for direct execution by the host. These functions must use `#[no_mangle]` and can interact with the host using `extism_pdk::input` and `output`.

```rust
#[no_mangle]
pub fn my_custom_task() -> i32 {
    // Read input from the host
    let input = extism_pdk::input::<String>().unwrap();
    
    // Process data...
    let result = format!("Processed: {}", input);
    
    // Return output back to the host
    extism_pdk::output(result);
    0 // Return code (0 for success)
}
```

## Host Functions

The WasmBridge host provides several "host functions" that your plugin can call. These are defined in the `extern "ExtismHost"` block:

*   `insecure_get(url)`: Fetches data via HTTP without SSL verification (useful for local self-signed services).
*   `get_date()`: Returns the current date from the host's system clock.

## Building

To build your plugin, run:

```bash
cargo build --target wasm32-unknown-unknown --release
```

The resulting `.wasm` file will be located in `target/wasm32-unknown-unknown/release/`.

## Deployment

Copy the compiled `.wasm` file to the `plugins` folder of your WasmBridge installation (usually `%APPDATA%\WasmBridge\plugins` on Windows). The host will automatically detect and load it.
