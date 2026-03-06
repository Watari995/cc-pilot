use anyhow::{anyhow, Result};
use log::{info, warn};
use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::session::{Environment, Session, SessionStatus};
use crate::settings;
use crate::watcher::SessionStore;

/// ポーリング間隔（秒）
const POLL_INTERVAL_SECS: u64 = 30;

/// 直近何日以内のセッションを取得するか
const RECENT_DAYS: i64 = 3;

/// Web セッション ID のプレフィックス（ローカルセッションとの衝突回避）
const WEB_ID_PREFIX: &str = "web_";

/// 前回取得した Web セッション ID を記録（削除検出用）
static PREVIOUS_WEB_IDS: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// Web セッション監視のポーリングを開始（別スレッドで実行）
pub fn start_polling(app_handle: AppHandle, store: SessionStore) -> Result<()> {
    std::thread::spawn(move || {
        if let Err(e) = polling_loop(&app_handle, &store) {
            log::error!("Web client polling error: {}", e);
        }
    });
    Ok(())
}

/// ポーリングループ本体
fn polling_loop(app_handle: &AppHandle, store: &SessionStore) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .build()?;

    let mut cached_session_key: Option<String> = None;
    let mut logged_sample = false;

    info!(
        "Web client polling started (interval: {}s)",
        POLL_INTERVAL_SECS
    );

    loop {
        std::thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));

        let settings = settings::load_settings(app_handle);
        let session_key = match &settings.claude_session_key {
            Some(key) if !key.is_empty() => key.clone(),
            _ => {
                clear_web_sessions(app_handle, store);
                cached_session_key = None;
                logged_sample = false;
                continue;
            }
        };

        if cached_session_key.as_ref() != Some(&session_key) {
            cached_session_key = Some(session_key.clone());
            logged_sample = false;
        }

        match fetch_sessions(&client, &session_key) {
            Ok(raw_sessions) => {
                if !logged_sample {
                    if let Some(first) = raw_sessions.first() {
                        info!(
                            "Web sessions API sample entry: {}",
                            serde_json::to_string_pretty(first).unwrap_or_default()
                        );
                    }
                    info!("Web sessions API returned {} entries", raw_sessions.len());
                    logged_sample = true;
                }
                process_sessions(app_handle, store, &raw_sessions);
            }
            Err(e) => {
                warn!("Failed to fetch sessions: {}", e);
                if e.to_string().contains("401") || e.to_string().contains("403") {
                    warn!("Session key may be expired");
                }
            }
        }
    }
}

/// Claude Code Web セッション一覧を取得
fn fetch_sessions(client: &Client, session_key: &str) -> Result<Vec<Value>> {
    let resp = client
        .get("https://claude.ai/v1/sessions")
        .header("Cookie", format!("sessionKey={}", session_key))
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-cache")
        .header("Anthropic-Beta", "ccr-byoc-2025-07-29")
        .header("Anthropic-Client-Feature", "ccr")
        .header("Anthropic-Client-Platform", "web_claude_ai")
        .header("Anthropic-Client-Version", "1.0.0")
        .header("Anthropic-Version", "2023-06-01")
        .send()?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        warn!(
            "Sessions API response body: {}",
            &body[..body.len().min(500)]
        );
        return Err(anyhow!("Sessions API returned {}", status));
    }

    let body: Value = resp.json()?;

    // レスポンス: {"data": [...], "first_id": "...", "has_more": false, "last_id": "..."}
    if let Some(Value::Array(arr)) = body.get("data") {
        Ok(arr.clone())
    } else if let Value::Array(arr) = body {
        Ok(arr)
    } else {
        warn!(
            "Unexpected sessions response structure: {}",
            serde_json::to_string(&body)
                .unwrap_or_default()
                .chars()
                .take(500)
                .collect::<String>()
        );
        Err(anyhow!("Unexpected response structure"))
    }
}

/// 取得したセッションをフィルタ・変換してセッションストアを更新
fn process_sessions(app_handle: &AppHandle, store: &SessionStore, raw_sessions: &[Value]) {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(RECENT_DAYS);
    let cutoff_str = cutoff.to_rfc3339();

    let mut current_web_ids = HashSet::new();

    for entry in raw_sessions {
        let session = match value_to_session(entry) {
            Some(s) => s,
            None => continue,
        };

        if session.last_activity_at < cutoff_str {
            continue;
        }

        current_web_ids.insert(session.id.clone());

        let mut sessions = store.lock().unwrap();
        sessions.insert(session.id.clone(), session.clone());
        drop(sessions);

        let _ = app_handle.emit("session-update", &session);
    }

    let mut prev = PREVIOUS_WEB_IDS.lock().unwrap();
    if let Some(prev_ids) = prev.as_ref() {
        for old_id in prev_ids.difference(&current_web_ids) {
            let mut sessions = store.lock().unwrap();
            sessions.remove(old_id);
            drop(sessions);

            let _ = app_handle.emit(
                "session-removed",
                serde_json::json!({ "id": old_id }),
            );
        }
    }
    *prev = Some(current_web_ids);
}

/// 生の JSON Value → Session 変換
fn value_to_session(v: &Value) -> Option<Session> {
    // session_id: "session_018z9r7eFeA2vBNDVe5vs1NY"
    let raw_id = v
        .get("session_id")
        .or_else(|| v.get("id"))
        .and_then(|v| v.as_str())?;

    let id = format!("{}{}", WEB_ID_PREFIX, raw_id);

    let title = v
        .get("name")
        .or_else(|| v.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();

    let created_at = v
        .get("created_at")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let updated_at = v
        .get("updated_at")
        .or_else(|| v.get("last_activity_at"))
        .and_then(|v| v.as_str())
        .unwrap_or(&created_at)
        .to_string();

    let model = v
        .get("model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let status = match chrono::DateTime::parse_from_rfc3339(&updated_at) {
        Ok(updated) => {
            let elapsed = chrono::Utc::now()
                .signed_duration_since(updated)
                .num_seconds();
            if elapsed < 300 {
                SessionStatus::Idle
            } else {
                SessionStatus::Done
            }
        }
        Err(_) => SessionStatus::Done,
    };

    Some(Session {
        id,
        project_path: String::new(),
        project_name: "claude.ai".to_string(),
        branch_name: None,
        title,
        alias: None,
        environment: Environment::Web,
        status,
        model,
        input_tokens: 0,
        output_tokens: 0,
        active_tools: Vec::new(),
        started_at: created_at,
        last_activity_at: updated_at,
        approval_detail: None,
        error_message: None,
    })
}

/// Web セッションを全てクリア（キー削除時）
fn clear_web_sessions(app_handle: &AppHandle, store: &SessionStore) {
    let mut sessions = store.lock().unwrap();
    let web_ids: Vec<String> = sessions
        .keys()
        .filter(|k| k.starts_with(WEB_ID_PREFIX))
        .cloned()
        .collect();
    for id in &web_ids {
        sessions.remove(id);
    }
    drop(sessions);

    for id in &web_ids {
        let _ = app_handle.emit("session-removed", serde_json::json!({ "id": id }));
    }

    let mut prev = PREVIOUS_WEB_IDS.lock().unwrap();
    *prev = None;
}
