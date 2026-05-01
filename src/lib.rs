#[cfg(not(target_arch = "wasm32"))]
compile_error!(
    "This project is intended for the wasm32-unknown-unknown architecture only. Please use: cargo build --target wasm32-unknown-unknown"
);

use extism_pdk::*;
use plugin_protocol::{
    EndpointInfo, PluginInfo, PluginRequest, PluginResponse, SettingDef, SettingType,
};

/// Definitions of host functions provided by the WasmBridge host.
/// You can call these functions from your plugin to interact with the host system.
#[host_fn]
extern "ExtismHost" {
    /// Performs an insecure HTTP GET request (bypassing SSL verification).
    fn insecure_get(url: String) -> Vec<u8>;
    /// Retrieves the current system date from the host.
    fn get_date() -> String;
}

/// REQUIRED: The `info` function defines your plugin's metadata.
/// The host calls this to discover available endpoints and settings.
#[plugin_fn]
pub fn info() -> FnResult<Json<PluginInfo>> {
    let info = PluginInfo {
        name: "example-plugin".into(),
        version: "0.1.0".into(),
        description: "An example Rust plugin for monitoring and data retrieval.".into(),
        
        // Define endpoints that will be available via the host's HTTP proxy.
        endpoints: vec![
            EndpointInfo {
                path: "/info".into(),
                method: "GET".into(),
                description: "Basic module information and current configuration".into(),
            },
            EndpointInfo {
                path: "/secure_data".into(),
                method: "GET".into(),
                description: "Retrieves data using a standard secure HTTP request (SSL verification)".into(),
            },
            EndpointInfo {
                path: "/insecure_data".into(),
                method: "GET".into(),
                description: "Retrieves data using an insecure host-provided helper (no SSL verification)".into(),
            },
        ],
        
        // Define settings that users can configure through the host UI.
        settings: vec![
            SettingDef {
                key: "api_url".into(),
                label: "API URL".into(),
                setting_type: SettingType::Text,
                default_value: Some("https://api.example.com".into()),
                description: Some("Base URL for external API requests".into()),
            },
            SettingDef {
                key: "api_key".into(),
                label: "API Key".into(),
                setting_type: SettingType::Password,
                default_value: None,
                description: Some("Secret authorization key for the API".into()),
            },
            SettingDef {
                key: "retry_count".into(),
                label: "Retry Count".into(),
                setting_type: SettingType::Number,
                default_value: Some("3".into()),
                description: None,
            },
            SettingDef {
                key: "enable_cache".into(),
                label: "Enable Caching".into(),
                setting_type: SettingType::Boolean,
                default_value: Some("true".into()),
                description: Some("Toggle local response caching".into()),
            },
        ],
    };
    Ok(Json(info))
}

/// REQUIRED: The `handle_request` function processes incoming HTTP requests routed by the host.
#[plugin_fn]
pub fn handle_request(Json(req): Json<PluginRequest>) -> FnResult<Json<PluginResponse>> {
    log!(LogLevel::Info, "Handling request: {} {}", req.method, req.path);
    
    // Route logic based on method and path
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/info") => {
            // Retrieve configuration values set by the user via the host.
            let api_url = config::get("api_url")?.unwrap_or_default();
            let enable_cache = config::get("enable_cache")?.unwrap_or_else(|| "false".into());

            let body = serde_json::to_vec(&serde_json::json!({
                "status": "active",
                "plugin": "example-plugin",
                "current_config": {
                    "api_url": api_url,
                    "enable_cache": enable_cache
                }
            }))?;

            Ok(Json(PluginResponse { status: 200, headers: Default::default(), body: Some(body) }))
        }
        ("GET", "/secure_data") => {
            let url = config::get("api_url")?.unwrap_or_default().trim().to_string();
            if url.is_empty() {
                return Ok(Json(PluginResponse {
                    status: 400,
                    headers: Default::default(),
                    body: Some(b"api_url is not configured".to_vec()),
                }));
            }

            log!(LogLevel::Info, "Sending secure request to: {}", url);
            // Use the standard Extism PDK HTTP client for secure requests.
            let http_req = HttpRequest::new(&url).with_method("GET");

            let resp = match http::request::<()>(&http_req, None) {
                Ok(r) => r,
                Err(e) => {
                    log!(LogLevel::Error, "Secure request failed: {}", e);
                    return Ok(Json(PluginResponse {
                        status: 502,
                        headers: Default::default(),
                        body: Some(format!("Secure request failed: {}", e).into_bytes()),
                    }));
                }
            };

            let status = resp.status_code();
            let body = resp.body();
            log!(
                LogLevel::Info,
                "Secure request successful, status: {}, body len: {}",
                status,
                body.len()
            );

            Ok(Json(PluginResponse {
                status: status as u16,
                headers: Default::default(),
                body: Some(body),
            }))
        }
        ("GET", "/insecure_data") => {
            let url = config::get("api_url")?.unwrap_or_default();
            if url.is_empty() {
                return Ok(Json(PluginResponse {
                    status: 400,
                    headers: Default::default(),
                    body: Some(b"api_url is not configured".to_vec()),
                }));
            }
            // Use a host function for specific tasks (like bypassing SSL).
            let body = unsafe { insecure_get(url)? };

            Ok(Json(PluginResponse { status: 200, headers: Default::default(), body: Some(body) }))
        }
        _ => Ok(Json(PluginResponse {
            status: 404,
            headers: Default::default(),
            body: Some(b"Not Found".to_vec()),
        })),
    }
}

/// OPTIONAL: `execute_command` handles tasks pushed from the Cloud Control Plane (Reverse Push).
#[plugin_fn]
pub fn execute_command(
    Json(payload): Json<plugin_protocol::CloudCommandPayload>,
) -> FnResult<Json<plugin_protocol::CloudCommandResult>> {
    log!(LogLevel::Info, "Received cloud command task: {}", payload.task);

    // Process tasks sent remotely from the cloud server.
    match payload.task.as_str() {
        "ping" => Ok(Json(plugin_protocol::CloudCommandResult {
            success: true,
            message: "pong".into(),
            data: None,
        })),
        "get_metrics" => {
            // Emulate gathering internal plugin metrics.
            let metrics = b"{\"cpu\": 45, \"ram\": 1024}".to_vec();
            Ok(Json(plugin_protocol::CloudCommandResult {
                success: true,
                message: "Metrics collected successfully".into(),
                data: Some(metrics),
            }))
        }
        _ => Ok(Json(plugin_protocol::CloudCommandResult {
            success: false,
            message: format!("Unknown task: {}", payload.task),
            data: None,
        })),
    }
}

/// A generic export for direct execution testing.
#[unsafe(no_mangle)]
pub fn test() -> i32 {
    // Read input from the host.
    let input = match extism_pdk::input::<String>() {
        Ok(s) => s,
        Err(_) => return 1,
    };

    // Use a host function to get the current date.
    let now = unsafe { get_date().unwrap_or_else(|_| "2026-05-01".to_string()) };
    let response = format!("Plugin received input: '{}'. Current agent date: {}", input, now);

    // Send output back to the host.
    extism_pdk::output(response);
    0
}
