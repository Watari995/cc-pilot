use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";
const ALIASES_KEY: &str = "session-aliases";

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub accent_color: String,
    pub terminal_app: String,
    pub launch_at_login: bool,
    pub notifications_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_session_key: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            accent_color: "#E8734A".to_string(),
            terminal_app: "ghostty".to_string(),
            launch_at_login: true,
            notifications_enabled: true,
            claude_session_key: None,
        }
    }
}

/// Store から設定を読み取る
pub fn load_settings(app: &AppHandle) -> Settings {
    let store = app.store(STORE_FILE).unwrap_or_else(|e| {
        log::warn!("Failed to open store, using defaults: {}", e);
        app.store_builder(STORE_FILE).build().unwrap()
    });

    store
        .get(SETTINGS_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Store に設定を保存する
pub fn save_settings(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let value = serde_json::to_value(settings).map_err(|e| e.to_string())?;
    store.set(SETTINGS_KEY, value);
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;
    Ok(())
}

/// セッションエイリアスを保存する
pub fn save_alias(
    app: &AppHandle,
    session_id: &str,
    alias: Option<&str>,
) -> Result<(), String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let mut aliases: std::collections::HashMap<String, String> = store
        .get(ALIASES_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    match alias {
        Some(a) if !a.is_empty() => {
            aliases.insert(session_id.to_string(), a.to_string());
        }
        _ => {
            aliases.remove(session_id);
        }
    }

    let value = serde_json::to_value(&aliases).map_err(|e| e.to_string())?;
    store.set(ALIASES_KEY, value);
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;
    Ok(())
}

/// セッションエイリアスを取得する
pub fn load_aliases(app: &AppHandle) -> std::collections::HashMap<String, String> {
    let store = app.store(STORE_FILE).unwrap_or_else(|e| {
        log::warn!("Failed to open store for aliases: {}", e);
        app.store_builder(STORE_FILE).build().unwrap()
    });

    store
        .get(ALIASES_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}
