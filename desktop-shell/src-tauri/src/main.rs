use serde::Serialize;
use tauri::{Manager, include_image};

const DESKTOP_ICON: tauri::image::Image<'_> = include_image!("./icons/icon.png");

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopRuntimeSnapshot {
    backend_base_url: String,
    database_url: String,
    redis_url: String,
    multipoint_enabled: bool,
}

fn env_flag(key: &str, default: bool) -> bool {
    std::env::var(key)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "True"))
        .unwrap_or(default)
}

fn desktop_debug_log(message: &str) {
    if env_flag("POP_TAIL_DESKTOP_LOG", cfg!(debug_assertions)) {
        eprintln!("[pop-tail-desktop] {message}");
    }
}

#[tauri::command]
fn desktop_toggle_fullscreen(window: tauri::WebviewWindow) -> Result<bool, String> {
    desktop_debug_log("toggle fullscreen requested");
    let next = !window.is_fullscreen().map_err(|err| err.to_string())?;
    window
        .set_fullscreen(next)
        .map_err(|err| err.to_string())?;
    desktop_debug_log(&format!("fullscreen changed: {next}"));
    Ok(next)
}

#[tauri::command]
fn desktop_is_fullscreen(window: tauri::WebviewWindow) -> Result<bool, String> {
    let is_fullscreen = window.is_fullscreen().map_err(|err| err.to_string())?;
    desktop_debug_log(&format!("fullscreen status read: {is_fullscreen}"));
    Ok(is_fullscreen)
}

#[tauri::command]
fn desktop_runtime_snapshot() -> DesktopRuntimeSnapshot {
    let snapshot = DesktopRuntimeSnapshot {
        backend_base_url: std::env::var("POP_TAIL_DESKTOP_BACKEND_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8888".to_string()),
        database_url: std::env::var("POP_TAIL_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://pop_tail:pop_tail_dev_secret@127.0.0.1:15432/pop_tail_auth".to_string()),
        redis_url: std::env::var("POP_TAIL_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:16379/0".to_string()),
        multipoint_enabled: std::env::var("POP_TAIL_MULTIPOINT_ENABLED")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "True"))
            .unwrap_or(true),
    };
    desktop_debug_log(&format!(
        "runtime snapshot requested: backend={}, redis={}, multipoint={}",
        snapshot.backend_base_url, snapshot.redis_url, snapshot.multipoint_enabled
    ));
    snapshot
}

fn main() {
    desktop_debug_log("starting desktop shell");
    tauri::Builder::default()
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_icon(DESKTOP_ICON.clone());
                desktop_debug_log("main window initialized");

                #[cfg(debug_assertions)]
                {
                    if env_flag("POP_TAIL_DESKTOP_OPEN_DEVTOOLS", cfg!(debug_assertions)) {
                        window.open_devtools();
                        desktop_debug_log("webview devtools opened");
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_runtime_snapshot,
            desktop_toggle_fullscreen,
            desktop_is_fullscreen
        ])
        .run(tauri::generate_context!())
        .expect("desktop shell failed to start");
}
