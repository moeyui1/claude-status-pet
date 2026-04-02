use notify::{EventKind, RecursiveMode, Watcher};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

#[derive(Clone, Serialize)]
struct StatusPayload {
    state: String,
    detail: String,
    tool: String,
    event: String,
    session_id: String,
    session_name: String,
}

fn default_status_path() -> PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".claude").join("pet-data").join("status.json")
}

fn read_status(path: &PathBuf) -> Option<StatusPayload> {
    let content = fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    Some(StatusPayload {
        state: v["state"].as_str().unwrap_or("idle").to_string(),
        detail: v["detail"].as_str().unwrap_or("").to_string(),
        tool: v["tool"].as_str().unwrap_or("").to_string(),
        event: v["event"].as_str().unwrap_or("").to_string(),
        session_id: v["session_id"].as_str().unwrap_or("").to_string(),
        session_name: v["session_name"].as_str().unwrap_or("").to_string(),
    })
}

#[tauri::command]
fn get_status(status_path: tauri::State<'_, Arc<Mutex<PathBuf>>>) -> Option<StatusPayload> {
    let path = status_path.lock().unwrap();
    read_status(&path)
}

#[tauri::command]
fn get_session_id(session_id: tauri::State<'_, String>) -> String {
    session_id.inner().clone()
}

#[tauri::command]
fn get_assets_dir(assets_dir: tauri::State<'_, Option<PathBuf>>) -> Option<String> {
    assets_dir.inner().as_ref().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn is_dlc_installed(assets_dir: tauri::State<'_, Option<PathBuf>>, dlc_name: String) -> bool {
    if let Some(dir) = assets_dir.inner().as_ref() {
        let dlc_dir = dir.join(&dlc_name);
        if let Ok(entries) = fs::read_dir(&dlc_dir) {
            return entries.filter_map(|e| e.ok()).any(|e| {
                e.path().extension().and_then(|ext| ext.to_str()) == Some("gif")
            });
        }
    }
    false
}

#[tauri::command]
fn download_dlc(assets_dir: tauri::State<'_, Option<PathBuf>>, dlc_name: String) -> Result<bool, String> {
    let dir = assets_dir.inner().as_ref().ok_or("No assets dir")?;

    // Try multiple script locations (JS preferred for cross-platform)
    let possible_scripts = vec![
        dir.join("../../scripts/download-gifs.js"),
        PathBuf::from(std::env::var("CLAUDE_PLUGIN_ROOT").unwrap_or_default()).join("scripts/download-gifs.js"),
    ];

    let script = possible_scripts.iter().find(|p| p.exists())
        .ok_or("download-gifs.js not found")?;

    let output = std::process::Command::new("node")
        .arg(script)
        .arg(dir.to_string_lossy().to_string())
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Download failed: {}", stderr));
    }

    Ok(true)
}

#[tauri::command]
fn load_asset(assets_dir: tauri::State<'_, Option<PathBuf>>, path: String) -> Option<String> {
    use base64::Engine;
    let dir = assets_dir.inner().as_ref()?;
    let file_path = dir.join(&path);
    let bytes = fs::read(&file_path).ok()?;
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
    let mime = match ext {
        "svg" => "image/svg+xml",
        "gif" => "image/gif",
        "png" => "image/png",
        _ => "application/octet-stream",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime, b64))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();

    let status_path = args
        .windows(2)
        .find(|w| w[0] == "--status-file")
        .map(|w| PathBuf::from(&w[1]))
        .unwrap_or_else(default_status_path);

    let session_id = args
        .windows(2)
        .find(|w| w[0] == "--session-id")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "unknown".to_string());

    let assets_dir: Option<PathBuf> = args
        .windows(2)
        .find(|w| w[0] == "--assets-dir")
        .map(|w| PathBuf::from(&w[1]));

    if let Some(parent) = status_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let status_path_shared = Arc::new(Mutex::new(status_path.clone()));
    let status_path_for_cleanup = status_path.clone();

    tauri::Builder::default()
        .manage(status_path_shared)
        .manage(session_id)
        .manage(assets_dir)
        .invoke_handler(tauri::generate_handler![get_status, get_session_id, get_assets_dir, load_asset, is_dlc_installed, download_dlc])
        .setup(move |app| {
            let window = app.get_webview_window("main").unwrap();

            // Set WebView2 background to transparent
            let _ = window.with_webview(|webview| {
                #[cfg(windows)]
                {
                    let controller = webview.controller();
                    unsafe {
                        use webview2_com::Microsoft::Web::WebView2::Win32::*;
                        let controller2: ICoreWebView2Controller2 =
                            windows::core::Interface::cast(&controller).unwrap();
                        let _ = controller2.SetDefaultBackgroundColor(COREWEBVIEW2_COLOR {
                            R: 0,
                            G: 0,
                            B: 0,
                            A: 0,
                        });
                    }
                }
            });

            let handle = app.handle().clone();
            let watch_path = status_path.clone();

            std::thread::spawn(move || {
                let (tx, rx) = std::sync::mpsc::channel();
                let mut watcher = notify::recommended_watcher(tx).unwrap();

                let watch_dir = watch_path.parent().unwrap().to_path_buf();
                let _ = fs::create_dir_all(&watch_dir);
                watcher
                    .watch(&watch_dir, RecursiveMode::NonRecursive)
                    .unwrap();

                if let Some(status) = read_status(&watch_path) {
                    let _ = handle.emit("status-update", status);
                }

                // Watch for file changes AND file deletion (session ended)
                for event in rx {
                    if let Ok(event) = event {
                        let is_our_file = event.paths.iter().any(|p| *p == watch_path);
                        if !is_our_file {
                            continue;
                        }
                        match event.kind {
                            EventKind::Modify(_) | EventKind::Create(_) => {
                                std::thread::sleep(std::time::Duration::from_millis(50));
                                if let Some(status) = read_status(&watch_path) {
                                    let _ = handle.emit("status-update", status);
                                }
                            }
                            EventKind::Remove(_) => {
                                // Session file deleted — session ended, close the pet
                                let _ = handle.emit(
                                    "status-update",
                                    StatusPayload {
                                        state: "closed".to_string(),
                                        detail: "Session ended".to_string(),
                                        tool: String::new(),
                                        event: "SessionEnd".to_string(),
                                        session_id: String::new(),
                                        session_name: String::new(),
                                    },
                                );
                            }
                            _ => {}
                        }
                    }
                }
            });

            Ok(())
        })
        .on_window_event(move |_window, event| {
            // Clean up status file when window is closed
            if let tauri::WindowEvent::Destroyed = event {
                let _ = fs::remove_file(&status_path_for_cleanup);
            }
        })
        .run(tauri::generate_context!())
        .expect("error running app");
}
