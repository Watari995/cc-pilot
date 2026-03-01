use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::session::Session;
use crate::settings;

/// セッションが needs_approval に遷移したときに macOS 通知を送信する
pub fn notify_approval_needed(app: &AppHandle, session: &Session) {
    let s = settings::load_settings(app);
    if !s.notifications_enabled {
        return;
    }

    let tool_name = session
        .approval_detail
        .as_ref()
        .map(|d| d.tool.as_str())
        .unwrap_or("unknown tool");

    let body = format!("{}: {}", session.project_name, tool_name);

    if let Err(e) = app
        .notification()
        .builder()
        .title("Approval Needed")
        .body(&body)
        .show()
    {
        log::warn!("Failed to send notification: {}", e);
    }
}
