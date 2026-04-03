use notify::{EventKind, RecursiveMode, Watcher};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

pub mod adapter;
pub mod status_map;
#[cfg(test)]
mod tests;

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

fn debug_log(path: &PathBuf, msg: &str) {
    if !DEBUG_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    // path can be a status file or the log file itself — resolve to pet-debug.log in same dir
    let log_path = if path.is_dir() {
        path.join("pet-debug.log")
    } else if path.file_name().map_or(false, |f| f == "pet-debug.log") {
        path.clone()
    } else {
        path.parent().unwrap_or(path).join("pet-debug.log")
    };
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| {
            let secs = d.as_secs();
            let millis = d.subsec_millis();
            let h = (secs % 86400) / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            format!("{:02}:{:02}:{:02}.{:03}", h, m, s, millis)
        })
        .unwrap_or_default();
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&log_path) {
        let _ = writeln!(f, "[{}] {}", timestamp, msg);
    }
}

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

    let gifs: Vec<(&str, &str)> = match dlc_name.as_str() {
        "mona" => vec![
            ("mona/love.gif",      "https://media.giphy.com/media/jrdgDVFrcgJpNlonWO/giphy.gif"),
            ("mona/angry.gif",     "https://media.giphy.com/media/kmCCrDo2vlIu6Kswop/giphy.gif"),
            ("mona/looking.gif",   "https://media.giphy.com/media/9f8mk4P3X2Nvch1z2o/giphy.gif"),
            ("mona/mona.gif",      "https://media.giphy.com/media/OFEabGCcVqsckIGn8G/giphy.gif"),
            ("mona/tongue.gif",    "https://media.giphy.com/media/WcYnTzdrjQphdu33xs/giphy.gif"),
            ("mona/shocked.gif",   "https://media.giphy.com/media/JdQFsdoJBcHaPOANdK/giphy.gif"),
            ("mona/smirk.gif",     "https://media.giphy.com/media/0vTOscboHgOyBSuK4r/giphy.gif"),
            ("mona/laugh.gif",     "https://media.giphy.com/media/RgutegYIHk2Nhxj4m5/giphy.gif"),
            ("mona/ohbrother.gif", "https://media.giphy.com/media/pMzEfC42AYlqT2WPaf/giphy.gif"),
            ("mona/hearts.gif",    "https://media.giphy.com/media/wJBYx2Yh84XS4sTzmz/giphy.gif"),
            ("mona/sick.gif",      "https://media.giphy.com/media/nfL2nlWacI8d9jgVXb/giphy.gif"),
            ("mona/tech.gif",      "https://media.giphy.com/media/cDZJ17fbzWVle68VCB/giphy.gif"),
            ("mona/ducks.gif",     "https://media.giphy.com/media/QxT6pLq6ekKiCkLkf0/giphy.gif"),
        ],
        "kuromi" => vec![
            ("kuromi/bling.gif",    "https://media.giphy.com/media/JNxq0xOWfidCDzqUH3/giphy.gif"),
            ("kuromi/charming.gif", "https://media.giphy.com/media/gkLG3Ki3OTXDwVb4rY/giphy.gif"),
            ("kuromi/kuromi.gif",   "https://media.giphy.com/media/MphoCSnXeA6wR4L8IS/giphy.gif"),
            ("kuromi/lilrya.gif",   "https://media.giphy.com/media/4G0nkrrXm8Xe1VgPzF/giphy.gif"),
            ("kuromi/jump.gif",     "https://media.giphy.com/media/cQSjIBgUC2NbMKEm9q/giphy.gif"),
            ("kuromi/sleeping.gif", "https://media.giphy.com/media/ZIskbLAG8Qeiq2dbV5/giphy.gif"),
            ("kuromi/heart.gif",    "https://media.giphy.com/media/VpCEcS3ZJ4qlKcD8LF/giphy.gif"),
            ("kuromi/think.gif",    "https://media.giphy.com/media/dCRVRbdbZUlNt1sRPd/giphy.gif"),
            ("kuromi/angry.gif",    "https://media.giphy.com/media/Qtvvgwbl1svKYcUGIT/giphy.gif"),
        ],
        _ => return Err(format!("Unknown DLC: {}", dlc_name)),
    };

    let dlc_dir = dir.join(&dlc_name);
    let _ = fs::create_dir_all(&dlc_dir);

    // Download each GIF using platform-native commands
    let mut failed = Vec::new();
    for (name, url) in &gifs {
        let dest = dir.join(name);
        let ok = if cfg!(windows) {
            std::process::Command::new("powershell")
                .args(["-Command", &format!(
                    "Invoke-WebRequest -Uri '{}' -OutFile '{}' -MaximumRedirection 5",
                    url, dest.to_string_lossy()
                )])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        } else {
            std::process::Command::new("curl")
                .args(["-sLo", &dest.to_string_lossy(), url])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        };
        if !ok {
            failed.push(*name);
        }
    }

    if !failed.is_empty() {
        return Err(format!("Failed to download: {}", failed.join(", ")));
    }

    // Write character.json
    let config = match dlc_name.as_str() {
        "mona" => serde_json::json!({
            "name": "Mona (GitHub)", "type": "gif",
            "states": {
                "idle": ["mona/love.gif", "mona/hearts.gif", "mona/smirk.gif"],
                "thinking": ["mona/looking.gif", "mona/mona.gif"],
                "reading": ["mona/mona.gif", "mona/looking.gif"],
                "editing": ["mona/tongue.gif", "mona/laugh.gif"],
                "searching": ["mona/tech.gif", "mona/looking.gif"],
                "running": ["mona/tongue.gif", "mona/tech.gif"],
                "delegating": ["mona/ducks.gif", "mona/smirk.gif"],
                "waiting": ["mona/shocked.gif", "mona/mona.gif"],
                "error": ["mona/angry.gif", "mona/sick.gif"],
                "offline": ["mona/ohbrother.gif", "mona/mona.gif"],
                "unknown": ["mona/love.gif"]
            }
        }),
        "kuromi" => serde_json::json!({
            "name": "Kuromi (Sanrio)", "type": "gif",
            "states": {
                "idle": ["kuromi/charming.gif", "kuromi/lilrya.gif"],
                "thinking": ["kuromi/think.gif", "kuromi/bling.gif"],
                "reading": ["kuromi/kuromi.gif"],
                "editing": ["kuromi/jump.gif", "kuromi/charming.gif"],
                "searching": ["kuromi/think.gif"],
                "running": ["kuromi/jump.gif", "kuromi/charming.gif"],
                "delegating": ["kuromi/heart.gif", "kuromi/lilrya.gif"],
                "waiting": ["kuromi/bling.gif", "kuromi/lilrya.gif"],
                "error": ["kuromi/angry.gif"],
                "offline": ["kuromi/sleeping.gif"],
                "unknown": ["kuromi/charming.gif"]
            }
        }),
        _ => unreachable!(),
    };

    let _ = fs::write(dlc_dir.join("character.json"), serde_json::to_string_pretty(&config).unwrap());

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

#[tauri::command]
fn load_text_asset(assets_dir: tauri::State<'_, Option<PathBuf>>, path: String) -> Option<String> {
    let dir = assets_dir.inner().as_ref()?;
    // Try assets dir first, then custom characters dir
    let asset_path = dir.join(&path);
    if let Ok(content) = fs::read_to_string(&asset_path) {
        return Some(content);
    }
    let custom_path = dir.parent()?.join("characters").join(&path);
    fs::read_to_string(&custom_path).ok()
}

#[tauri::command]
fn load_custom_asset(assets_dir: tauri::State<'_, Option<PathBuf>>, path: String) -> Option<String> {
    use base64::Engine;
    let dir = assets_dir.inner().as_ref()?;
    // Try assets dir, then custom characters dir
    let file_path = dir.join(&path);
    let file_path = if file_path.exists() { file_path } else { dir.parent()?.join("characters").join(&path) };
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

#[derive(Clone, Serialize)]
struct CharacterPack {
    id: String,
    name: String,
    #[serde(rename = "type")]
    char_type: String,
    group: String,
    installed: bool,
    config_path: String,
}

#[tauri::command]
fn list_character_packs(assets_dir: tauri::State<'_, Option<PathBuf>>) -> Vec<CharacterPack> {
    let mut packs = Vec::new();

    if let Some(dir) = assets_dir.inner().as_ref() {
        // Scan assets dir (DLC: mona, kuromi, etc.)
        scan_packs_in_dir(dir, "dlc", &mut packs);

        // Scan custom characters dir (sibling to assets)
        let custom_dir = dir.parent().map(|p| p.join("characters")).unwrap_or_default();
        if custom_dir.is_dir() {
            scan_packs_in_dir(&custom_dir, "custom", &mut packs);
        }
    }

    packs
}

fn scan_packs_in_dir(dir: &PathBuf, group: &str, packs: &mut Vec<CharacterPack>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let config_path = path.join("character.json");
            let id = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    packs.push(CharacterPack {
                        id: id.clone(),
                        name: v["name"].as_str().unwrap_or(&id).to_string(),
                        char_type: v["type"].as_str().unwrap_or("gif").to_string(),
                        group: group.to_string(),
                        installed: true,
                        config_path: config_path.to_string_lossy().to_string(),
                    });
                }
            } else {
                // Directory exists but no character.json — check if has image files
                let has_images = fs::read_dir(&path).ok().map_or(false, |entries| {
                    entries.filter_map(|e| e.ok()).any(|e| {
                        let ext = e.path().extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
                        ext == "gif" || ext == "svg" || ext == "png"
                    })
                });
                if has_images {
                    packs.push(CharacterPack {
                        id,
                        name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                        char_type: "gif".to_string(),
                        group: group.to_string(),
                        installed: true,
                        config_path: String::new(),
                    });
                }
            }
        }
    }
}

/// CLI: write-status subcommand
/// Reads event info from CLI args or stdin (via adapter), writes status JSON, exits.
fn cmd_write_status(args: &[String]) {
    // Enable debug if --debug is passed
    if args.iter().any(|a| a == "--debug") {
        DEBUG_ENABLED.store(true, Ordering::Relaxed);
    }
    let pet_dir = default_pet_dir();
    let _ = fs::create_dir_all(&pet_dir);
    let log_path = pet_dir.join("pet-debug.log");  // dummy file path for debug_log
    let t0 = std::time::Instant::now();

    debug_log(&log_path, &format!("write-status START args={:?}", &args[1..]));

    // Parse CLI args
    let adapter_name = get_arg(args, "--adapter");
    let event_arg = get_arg(args, "--event");
    let tool_arg = get_arg(args, "--tool").unwrap_or_default();
    let detail_arg = get_arg(args, "--detail").unwrap_or_default();
    let session_id_arg = get_arg(args, "--session-id");
    let session_name_arg = get_arg(args, "--session-name");

    let (event, tool, detail, session_id, session_name, launch_only) =
        if let Some(adapter_name) = &adapter_name {
            // Adapter mode: read stdin JSON
            debug_log(&log_path, "reading stdin...");
            let stdin_data = read_stdin();
            debug_log(&log_path, &format!("stdin read in {:?}, len={}, data={}", t0.elapsed(), stdin_data.len(), &stdin_data));
            let stdin: adapter::StdinInput = serde_json::from_str(&stdin_data).unwrap_or_default();

            if let Some(adapter) = adapter::get_adapter(adapter_name) {
                if let Some(ev) = adapter.parse(&stdin) {
                    (ev.event, ev.tool, ev.detail, ev.session_id, ev.session_name, ev.launch_only)
                } else {
                    return;
                }
            } else {
                eprintln!("Unknown adapter: {}", adapter_name);
                std::process::exit(1);
            }
        } else if let Some(event) = event_arg {
            // CLI args mode
            let sid = session_id_arg.unwrap_or_else(|| "cli".to_string());
            let sname = session_name_arg.unwrap_or_else(|| sid.clone());
            let detail = if detail_arg.is_empty() {
                status_map::tool_detail(&tool_arg, "", "")
            } else {
                detail_arg
            };
            (event, tool_arg, detail, sid, sname, false)
        } else {
            eprintln!("Usage: claude-status-pet write-status --event <prompt|tool|done|error|offline> [--tool <name>] [--detail <text>] [--session-id <id>]");
            eprintln!("   or: claude-status-pet write-status --adapter <claude|copilot> < stdin.json");
            std::process::exit(1);
        };

    // Determine state from event + tool
    let state = if event == "tool" && !tool.is_empty() {
        status_map::tool_to_state(&tool)
    } else {
        status_map::event_to_state(&event)
    };

    let status_file = pet_dir.join(format!("status-{}.json", session_id));

    debug_log(&log_path, &format!("writing state={} tool={} to {:?}", state, tool, status_file));

    // launch_only: don't write status (e.g. Copilot sessionStart — GUI launch handled by hook script)
    if launch_only {
        debug_log(&log_path, "launch_only — skipping status write");
    } else {
        let status = serde_json::json!({
            "state": state, "detail": detail, "tool": tool,
            "event": event, "session_id": session_id,
            "session_name": session_name, "timestamp": timestamp()
        });
        let _ = fs::write(&status_file, status.to_string());
    }

    debug_log(&log_path, &format!("file written in {:?}", t0.elapsed()));
    debug_log(&log_path, &format!("write-status DONE in {:?}", t0.elapsed()));
}

fn cleanup_stale_status(pet_dir: &PathBuf) {
    let Ok(entries) = fs::read_dir(pet_dir) else { return };
    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(24 * 3600);
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("status-") && name_str.ends_with(".json") {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified < cutoff {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}

fn read_stdin() -> String {
    use std::io::Read;
    // Read stdin until complete JSON object (depth-balanced {}) or timeout.
    // Does NOT wait for EOF — returns as soon as JSON is complete.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escape = false;
        let mut started = false;

        for byte in std::io::stdin().bytes() {
            let Ok(b) = byte else { break };
            buf.push(b);

            // JSON structure tracking (only for ASCII control chars, safe for UTF-8
            // since multi-byte sequences never contain bytes < 0x80)
            if escape { escape = false; continue; }
            if b == b'\\' && in_string { escape = true; continue; }
            if b == b'"' { in_string = !in_string; continue; }
            if in_string { continue; }
            if b == b'{' { depth += 1; started = true; }
            if b == b'}' {
                depth -= 1;
                if started && depth == 0 {
                    let _ = tx.send(String::from_utf8_lossy(&buf).into_owned());
                    return;
                }
            }
        }
        let _ = tx.send(String::from_utf8_lossy(&buf).into_owned());
    });
    rx.recv_timeout(std::time::Duration::from_millis(100)).unwrap_or_default()
}

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone())
}

fn default_pet_dir() -> PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".claude").join("pet-data")
}

fn timestamp() -> String {
    // ISO 8601 UTC timestamp
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs();
    // Simple UTC format without chrono dependency
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let h = time_secs / 3600;
    let m = (time_secs % 3600) / 60;
    let s = time_secs % 60;
    // Approximate date (good enough for timestamps)
    let y = 1970 + days / 365;
    let remaining = days % 365;
    let month = remaining / 30 + 1;
    let day = remaining % 30 + 1;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, month, day, h, m, s)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();

    // Subcommand dispatch: write-status runs without GUI
    if args.iter().any(|a| a == "write-status") {
        cmd_write_status(&args);
        std::process::exit(0); // Force exit — don't wait for stdin reader thread
    }

    if args.iter().any(|a| a == "--debug") {
        DEBUG_ENABLED.store(true, Ordering::Relaxed);
    }

    let demo_mode = args.iter().any(|a| a == "--demo");

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

    // Write per-session PID lock file
    let pet_dir = default_pet_dir();
    let _ = fs::create_dir_all(&pet_dir);

    let assets_dir: Option<PathBuf> = args
        .windows(2)
        .find(|w| w[0] == "--assets-dir")
        .map(|w| PathBuf::from(&w[1]));

    if let Some(parent) = status_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Clean up stale status files on GUI startup
    cleanup_stale_status(&default_pet_dir());

    let status_path_shared = Arc::new(Mutex::new(status_path.clone()));
    let status_path_for_cleanup = status_path.clone();

    tauri::Builder::default()
        .manage(status_path_shared)
        .manage(session_id)
        .manage(assets_dir)
        .invoke_handler(tauri::generate_handler![get_status, get_session_id, get_assets_dir, load_asset, load_text_asset, load_custom_asset, is_dlc_installed, download_dlc, list_character_packs])
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
            let log_path = status_path.clone();

            if demo_mode {
                // Demo mode: cycle through all states for recording
                std::thread::spawn(move || {
                    let demos = vec![
                        ("idle", "Waiting for input..."),
                        ("thinking", "Processing prompt..."),
                        ("reading", "Reading lib.rs"),
                        ("editing", "Editing app.js"),
                        ("searching", "Searching: TODO"),
                        ("running", "Running npm test"),
                        ("delegating", "Spawning agent..."),
                        ("waiting", "Waiting for response..."),
                        ("error", "Build failed"),
                        ("offline", "Zzz..."),
                    ];
                    let mut i = 0;
                    loop {
                        let (state, detail) = demos[i % demos.len()];
                        let _ = handle.emit("status-update", StatusPayload {
                            state: state.to_string(),
                            detail: detail.to_string(),
                            tool: String::new(),
                            event: "Demo".to_string(),
                            session_id: "demo".to_string(),
                            session_name: "Demo Mode".to_string(),
                        });
                        i += 1;
                        std::thread::sleep(std::time::Duration::from_millis(1500));
                    }
                });
            } else {
                // Normal mode: watch status file
                std::thread::spawn(move || {
                let (tx, rx) = std::sync::mpsc::channel();
                let mut watcher = match notify::recommended_watcher(tx) {
                    Ok(w) => w,
                    Err(e) => {
                        debug_log(&log_path, &format!("FATAL: watcher init failed: {}", e));
                        return;
                    }
                };

                let watch_dir = match watch_path.parent() {
                    Some(p) => p.to_path_buf(),
                    None => {
                        debug_log(&log_path, "FATAL: status path has no parent dir");
                        return;
                    }
                };
                let _ = fs::create_dir_all(&watch_dir);
                if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
                    debug_log(&log_path, &format!("FATAL: watch() failed: {}", e));
                    return;
                }

                debug_log(&log_path, &format!("Watcher started on {:?}", watch_dir));

                if let Some(status) = read_status(&watch_path) {
                    debug_log(&log_path, &format!("Initial status: state={}", status.state));
                    let _ = handle.emit("status-update", status);
                }

                for event in rx {
                    if let Ok(event) = event {
                        let is_our_file = event.paths.iter().any(|p| *p == watch_path);
                        if !is_our_file {
                            continue;
                        }
                        debug_log(&log_path, &format!("Event: {:?}, paths: {:?}", event.kind, event.paths));
                        match event.kind {
                            EventKind::Modify(_) | EventKind::Create(_) => {
                                std::thread::sleep(std::time::Duration::from_millis(50));
                                if let Some(status) = read_status(&watch_path) {
                                    debug_log(&log_path, &format!("Emit: state={}, detail={}", status.state, status.detail));
                                    let _ = handle.emit("status-update", status);
                                } else {
                                    debug_log(&log_path, "Read failed after Modify/Create (file may be mid-write)");
                                }
                            }
                            EventKind::Remove(_) => {
                                // On Windows, writeFileSync can trigger Remove+Create.
                                // Wait briefly and check if file reappears before closing.
                                debug_log(&log_path, "Remove detected, waiting 300ms to confirm deletion...");
                                std::thread::sleep(std::time::Duration::from_millis(300));
                                if watch_path.exists() {
                                    debug_log(&log_path, "File still exists after Remove — spurious event (Windows writeFileSync), reading status");
                                    if let Some(status) = read_status(&watch_path) {
                                        debug_log(&log_path, &format!("Emit after spurious Remove: state={}, detail={}", status.state, status.detail));
                                        let _ = handle.emit("status-update", status);
                                    }
                                } else {
                                    debug_log(&log_path, "File truly deleted — emitting closed state");
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
                            }
                            _ => {
                                debug_log(&log_path, &format!("Ignored event: {:?}", event.kind));
                            }
                        }
                    } else if let Err(e) = event {
                        debug_log(&log_path, &format!("Watcher error: {:?}", e));
                    }
                }
                debug_log(&log_path, "Watcher loop ended (channel closed)");
                });
            } // end if demo_mode / else

            Ok(())
        })
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let _ = fs::remove_file(&status_path_for_cleanup);
            }
        })
        .run(tauri::generate_context!())
        .expect("error running app");
}
