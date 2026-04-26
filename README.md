# Шаблон плагина на Rust (WASM)

Данный шаблон предназначен для создания модулей интеграции для `monitoring-bridge`.

## Требования

1. Установленный Rust.
2. WASM target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

## Сборка

Для сборки плагина выполните:

```bash
cargo build --target wasm32-unknown-unknown --release
```

После сборки файл будет находиться по пути:
`target/wasm32-unknown-unknown/release/rust_plugin_template.wasm`

## Структура

- `info()`: Возвращает метаданные плагина.
- `handle_request()`: Основной обработчик запросов от системы.
