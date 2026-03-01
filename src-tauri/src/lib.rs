mod parser;
mod session;
mod watcher;

use session::Session;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 全セッション一覧を取得
#[tauri::command]
fn get_sessions(
    store: tauri::State<'_, watcher::SessionStore>,
) -> Result<Vec<Session>, String> {
    let sessions = store.lock().map_err(|e| e.to_string())?;
    let mut list: Vec<Session> = sessions.values().cloned().collect();
    // 最終アクティビティが新しい順にソート
    list.sort_by(|a, b| b.last_activity_at.cmp(&a.last_activity_at));
    Ok(list)
}

/// 環境に応じてアプリケーションを起動（ジャンプ機能）
#[tauri::command]
fn open_in_environment(session_id: String) -> Result<(), String> {
    // Phase 2 で実装予定
    log::info!("open_in_environment called for session: {}", session_id);
    Ok(())
}

/// アプリケーションの起動
pub fn run() {
    env_logger::init();

    let session_store: watcher::SessionStore = Arc::new(Mutex::new(HashMap::new()));

    // 初回スキャン
    if let Err(e) = watcher::initial_scan(&session_store) {
        log::error!("Initial scan failed: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(session_store.clone())
        .invoke_handler(tauri::generate_handler![get_sessions, open_in_environment])
        .setup(move |app| {
            let handle = app.handle().clone();
            // ファイル監視を開始
            if let Err(e) = watcher::start_watching(handle, session_store.clone()) {
                log::error!("Failed to start file watcher: {}", e);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
