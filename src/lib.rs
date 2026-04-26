#[cfg(not(target_arch = "wasm32"))]
compile_error!(
    "Этот проект предназначен только для архитектуры wasm32. Пожалуйста, используйте --target wasm32-unknown-unknown."
);

use extism_pdk::*;
use plugin_protocol::{
    EndpointInfo, PluginInfo, PluginRequest, PluginResponse, SettingDef, SettingType,
};
use serde_json;

#[host_fn]
extern "ExtismHost" {
    fn insecure_get(url: String) -> Vec<u8>;
}

#[plugin_fn]
pub fn info() -> FnResult<Json<PluginInfo>> {
    let info = PluginInfo {
        name: "example-plugin".into(),
        version: "0.1.0".into(),
        description: "Пример плагина на Rust для мониторинга".into(),
        endpoints: vec![
            EndpointInfo {
                path: "/info".into(),
                method: "GET".into(),
                description: "Базовая информация о модуле".into(),
            },
            EndpointInfo {
                path: "/secure_data".into(),
                method: "GET".into(),
                description: "Безопасный запрос (с проверкой SSL)".into(),
            },
            EndpointInfo {
                path: "/insecure_data".into(),
                method: "GET".into(),
                description: "Небезопасный запрос (без проверки SSL)".into(),
            },
        ],
        settings: vec![
            SettingDef {
                key: "api_url".into(),
                label: "URL API".into(),
                setting_type: SettingType::Text,
                default_value: Some("https://api.example.com".into()),
                description: Some("Базовый адрес для запросов данных".into()),
            },
            SettingDef {
                key: "api_key".into(),
                label: "API Ключ".into(),
                setting_type: SettingType::Password,
                default_value: None,
                description: Some("Секретный ключ авторизации".into()),
            },
            SettingDef {
                key: "retry_count".into(),
                label: "Кол-во попыток".into(),
                setting_type: SettingType::Number,
                default_value: Some("3".into()),
                description: None,
            },
            SettingDef {
                key: "enable_cache".into(),
                label: "Кэширование".into(),
                setting_type: SettingType::Boolean,
                default_value: Some("true".into()),
                description: Some("Включить локальное кэширование ответов".into()),
            },
        ],
    };
    Ok(Json(info))
}

#[plugin_fn]
pub fn handle_request(Json(req): Json<PluginRequest>) -> FnResult<Json<PluginResponse>> {
    // В зависимости от req.path и req.method выполняем логику
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/info") => {
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
            let url = config::get("api_url")?.unwrap_or_default();
            if url.is_empty() {
                return Ok(Json(PluginResponse {
                    status: 400,
                    headers: Default::default(),
                    body: Some(b"api_url is not configured".to_vec()),
                }));
            }

            let req = HttpRequest::new(&url).with_method("GET");
            let resp = match http::request::<()>(&req, None) {
                Ok(r) => r,
                Err(e) => {
                    return Ok(Json(PluginResponse {
                        status: 502,
                        headers: Default::default(),
                        body: Some(
                            format!("Secure request failed (SSL Error): {}", e).into_bytes(),
                        ),
                    }));
                }
            };

            Ok(Json(PluginResponse {
                status: resp.status_code() as u16,
                headers: Default::default(),
                body: Some(resp.body()),
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
