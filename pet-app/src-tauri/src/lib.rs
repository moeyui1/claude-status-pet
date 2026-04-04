use notify::{EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
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

fn init_debug(args: &[String]) {
    let _ = args; // reserved for future use
    if std::env::var("PET_DEBUG").map_or(false, |v| v == "1" || v == "true") {
        DEBUG_ENABLED.store(true, Ordering::Relaxed);
    }
}

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
fn get_session_id(session_id: tauri::State<'_, Arc<Mutex<String>>>) -> String {
    session_id.lock().unwrap().clone()
}

#[tauri::command]
fn get_assets_dir(assets_dir: tauri::State<'_, Option<PathBuf>>) -> Option<String> {
    assets_dir.inner().as_ref().map(|p| p.to_string_lossy().to_string())
}

#[derive(Clone, Serialize, Deserialize)]
struct SessionInfo {
    session_id: String,
    session_name: String,
    state: String,
    detail: String,
    status_file: String,
    last_modified: u64,
}

#[tauri::command]
fn list_unlocked_sessions() -> Vec<SessionInfo> {
    let pet_dir = default_pet_dir();
    let Ok(entries) = fs::read_dir(&pet_dir) else { return vec![] };
    let mut sessions = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();
        if name_str.starts_with("status-") && name_str.ends_with(".json") {
            let sid = name_str.strip_prefix("status-").unwrap().strip_suffix(".json").unwrap().to_string();
            let lock_file = pet_dir.join(format!("pet-{}.lock", sid));
            if is_lock_alive(&lock_file) {
                continue; // already has a running pet
            }
            // Clean up dead lock file
            if lock_file.exists() {
                let _ = fs::remove_file(&lock_file);
            }
            let status_path = entry.path();
            let last_modified = status_path.metadata()
                .and_then(|m| m.modified())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64)
                .unwrap_or(0);
            let (sname, state, detail) = if let Some(s) = read_status(&status_path) {
                (s.session_name, s.state, s.detail)
            } else {
                (String::new(), "idle".to_string(), String::new())
            };
            sessions.push(SessionInfo {
                session_id: sid,
                session_name: sname,
                state,
                detail,
                status_file: status_path.to_string_lossy().to_string(),
                last_modified,
            });
        }
    }
    sessions
}

#[tauri::command]
fn bind_session(
    session_id: String,
    status_path_state: tauri::State<'_, Arc<Mutex<PathBuf>>>,
    session_id_state: tauri::State<'_, Arc<Mutex<String>>>,
    lock_path_state: tauri::State<'_, Arc<Mutex<Option<PathBuf>>>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if !is_safe_session_id(&session_id) {
        return Err("Invalid session ID".to_string());
    }
    let pet_dir = default_pet_dir();
    let status_file = pet_dir.join(format!("status-{}.json", session_id));
    let lock_file = pet_dir.join(format!("pet-{}.lock", session_id));

    if is_lock_alive(&lock_file) {
        return Err("Session already has a running pet".to_string());
    }
    write_lock_file(&lock_file);

    // Update shared state
    *status_path_state.lock().unwrap() = status_file.clone();
    *session_id_state.lock().unwrap() = session_id.clone();
    *lock_path_state.lock().unwrap() = Some(lock_file);

    // Emit initial status
    if let Some(status) = read_status(&status_file) {
        let _ = app.emit("status-update", status);
    }

    // Start file watcher in a background thread
    let watch_path = status_file.clone();
    let log_path = status_file.clone();
    let handle = app.clone();
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
            None => return,
        };
        let _ = fs::create_dir_all(&watch_dir);
        if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
            debug_log(&log_path, &format!("FATAL: watch() failed: {}", e));
            return;
        }
        debug_log(&log_path, &format!("Watcher started on {:?} (bind_session)", watch_dir));

        for event in rx {
            if let Ok(event) = event {
                let is_our_file = event.paths.iter().any(|p| *p == watch_path);
                if !is_our_file { continue; }
                debug_log(&log_path, &format!("bind_session event: {:?}, paths: {:?}", event.kind, event.paths));
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        if let Some(status) = read_status(&watch_path) {
                            debug_log(&log_path, &format!("bind_session emit: state={}, detail={}", status.state, status.detail));
                            let _ = handle.emit("status-update", status);
                        }
                    }
                    EventKind::Remove(_) => {
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        if watch_path.exists() {
                            if let Some(status) = read_status(&watch_path) {
                                let _ = handle.emit("status-update", status);
                            }
                        } else {
                            let _ = handle.emit("status-update", StatusPayload {
                                state: "closed".to_string(),
                                detail: "Session ended".to_string(),
                                tool: String::new(),
                                event: "SessionEnd".to_string(),
                                session_id: String::new(),
                                session_name: String::new(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    Ok(())
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
async fn download_dlc(assets_dir: tauri::State<'_, Option<PathBuf>>, dlc_name: String) -> Result<bool, String> {
    let dir = assets_dir.inner().as_ref().ok_or("No assets dir")?.clone();
    let dlc = dlc_name.clone();

    tauri::async_runtime::spawn_blocking(move || {
        download_dlc_blocking(&dir, &dlc)
    }).await.map_err(|e| e.to_string())?
}

fn download_dlc_blocking(dir: &PathBuf, dlc_name: &str) -> Result<bool, String> {
    if !is_safe_session_id(dlc_name) {
        return Err("Invalid DLC name".to_string());
    }

    // Read DLC config from dlc/<name>.json
    let config_path = dir.join("dlc").join(format!("{}.json", dlc_name));
    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| format!("DLC config not found: {}: {}", config_path.display(), e))?;
    let config: serde_json::Value = serde_json::from_str(&config_str)
        .map_err(|e| format!("Invalid DLC config: {}", e))?;

    let downloads = config["downloads"].as_array()
        .ok_or_else(|| "DLC config missing 'downloads' array".to_string())?;

    let dlc_dir = dir.join(dlc_name);
    fs::create_dir_all(&dlc_dir)
        .map_err(|e| format!("Failed to create DLC directory: {}", e))?;

    // Download each file (validate paths stay within assets dir)
    let dir_canonical = dir.canonicalize().map_err(|e| format!("Invalid assets dir: {}", e))?;
    let mut failed = Vec::new();
    for item in downloads {
        let path = item["path"].as_str().unwrap_or("");
        let url = item["url"].as_str().unwrap_or("");
        if path.is_empty() || url.is_empty() { continue; }
        // Path traversal prevention: ensure dest stays within assets dir
        let dest = dir.join(path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory for {}: {}", path, e))?;
        }
        let dest_parent = dest.parent()
            .and_then(|p| p.canonicalize().ok())
            .ok_or_else(|| format!("Invalid path: {}", path))?;
        if !dest_parent.starts_with(&dir_canonical) {
            return Err(format!("Path traversal blocked: {}", path));
        }
        match download_file(url, &dest) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to download {}: {}", path, e);
                failed.push(path.to_string());
            }
        }
    }

    if !failed.is_empty() {
        return Err(format!("Failed to download: {}", failed.join(", ")));
    }

    // Write character.json from the states in the DLC config
    let character = serde_json::json!({
        "name": config["name"],
        "type": config["type"],
        "states": config["states"]
    });
    fs::write(dlc_dir.join("character.json"), serde_json::to_string_pretty(&character).unwrap())
        .map_err(|e| format!("Failed to write character.json: {}", e))?;

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
struct DlcInfo {
    id: String,
    name: String,
    installed: bool,
}

#[tauri::command]
fn list_available_dlcs(assets_dir: tauri::State<'_, Option<PathBuf>>) -> Vec<DlcInfo> {
    let mut dlcs = Vec::new();

    // Scan dlc/*.json in assets dir, with fallback to default pet-data/assets/dlc/
    let dirs_to_check: Vec<PathBuf> = if let Some(dir) = assets_dir.inner().as_ref() {
        vec![dir.clone()]
    } else {
        vec![default_pet_dir().join("assets")]
    };

    for dir in &dirs_to_check {
        let dlc_dir = dir.join("dlc");
        let Ok(entries) = fs::read_dir(&dlc_dir) else { continue };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }
            let id = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            if dlcs.iter().any(|d: &DlcInfo| d.id == id) { continue; }
            let Ok(content) = fs::read_to_string(&path) else { continue };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else { continue };
            let name = v["name"].as_str().unwrap_or(&id).to_string();
            let installed = dir.join(&id).join("character.json").exists();
            dlcs.push(DlcInfo { id, name, installed });
        }
    }
    dlcs
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
    init_debug(args);
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

    if !is_safe_session_id(&session_id) {
        eprintln!("Invalid session ID");
        std::process::exit(1);
    }

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

    // Output session info to stdout (hook script captures this to launch GUI)
    println!("{}\t{}", status_file.to_string_lossy(), session_id);

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
        // Also clean up orphaned lock files whose process is no longer alive
        if name_str.starts_with("pet-") && name_str.ends_with(".lock") {
            if !is_lock_alive(&entry.path().to_path_buf()) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

fn is_safe_session_id(id: &str) -> bool {
    !id.is_empty() && !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

fn write_lock_file(lock_path: &PathBuf) {
    let _ = fs::write(lock_path, std::process::id().to_string());
}

fn is_lock_alive(lock_path: &PathBuf) -> bool {
    let Ok(content) = fs::read_to_string(lock_path) else { return false };
    let Ok(pid) = content.trim().parse::<u32>() else { return false };
    is_process_running(pid)
}

#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    const STILL_ACTIVE: u32 = 259;
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }
        let mut exit_code: u32 = 0;
        let result = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);
        result != 0 && exit_code == STILL_ACTIVE
    }
}

#[cfg(windows)]
unsafe extern "system" {
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
    fn GetExitCodeProcess(handle: *mut std::ffi::c_void, exit_code: *mut u32) -> i32;
    fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
}

#[cfg(not(windows))]
fn is_process_running(pid: u32) -> bool {
    // signal 0 checks if process exists without sending a signal
    unsafe { libc_kill(pid as i32, 0) == 0 }
}

#[cfg(not(windows))]
unsafe extern "C" {
    #[link_name = "kill"]
    fn libc_kill(pid: i32, sig: i32) -> i32;
}

fn download_file(url: &str, dest: &PathBuf) -> Result<(), String> {
    use std::io::Read;
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(30)))
        .build()
        .new_agent();
    let resp = agent.get(url).call().map_err(|e| e.to_string())?;
    let status = resp.status();
    if status != 200 {
        return Err(format!("HTTP {}", status));
    }
    let mut bytes: Vec<u8> = Vec::new();
    resp.into_body().into_reader().read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.is_empty() {
        return Err("Empty response".into());
    }
    fs::write(dest, &bytes).map_err(|e| e.to_string())
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

    if std::env::var("PET_DEBUG").map_or(false, |v| v == "1" || v == "true") {
        DEBUG_ENABLED.store(true, Ordering::Relaxed);
    }

    let demo_mode = args.iter().any(|a| a == "--demo");

    let explicit_status = args.windows(2).find(|w| w[0] == "--status-file").map(|w| PathBuf::from(&w[1]));
    let explicit_session = args.windows(2).find(|w| w[0] == "--session-id").map(|w| w[1].clone());

    // Determine if we have an explicit session or need session selection
    let (initial_status_path, initial_session_id, initial_lock) = if let Some(sf) = &explicit_status {
        let sid = explicit_session.clone().unwrap_or_else(|| "unknown".to_string());
        let pet_dir = default_pet_dir();
        let _ = fs::create_dir_all(&pet_dir);
        let lock_file = pet_dir.join(format!("pet-{}.lock", sid));
        if is_lock_alive(&lock_file) {
            eprintln!("Pet already running for session {}", sid);
            std::process::exit(0);
        }
        write_lock_file(&lock_file);
        (sf.clone(), sid, Some(lock_file))
    } else {
        // No explicit session — will show session picker in frontend
        (default_status_path(), String::new(), None)
    };

    let needs_session_select = explicit_status.is_none() && !demo_mode;

    let assets_dir: Option<PathBuf> = args
        .windows(2)
        .find(|w| w[0] == "--assets-dir")
        .map(|w| PathBuf::from(&w[1]));

    if let Some(parent) = initial_status_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Clean up stale status files on GUI startup
    cleanup_stale_status(&default_pet_dir());

    let status_path_shared = Arc::new(Mutex::new(initial_status_path.clone()));
    let session_id_shared = Arc::new(Mutex::new(initial_session_id.clone()));
    let lock_path_shared: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(initial_lock.clone()));

    let lock_for_cleanup = lock_path_shared.clone();

    tauri::Builder::default()
        .manage(status_path_shared)
        .manage(session_id_shared)
        .manage(lock_path_shared)
        .manage(assets_dir)
        .invoke_handler(tauri::generate_handler![get_status, get_session_id, get_assets_dir, load_asset, load_text_asset, load_custom_asset, is_dlc_installed, download_dlc, list_available_dlcs, list_character_packs, list_unlocked_sessions, bind_session])
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

            if needs_session_select {
                // Frontend will query for session selection on init
                // (emit is unreliable here — JS may not have loaded listeners yet)
            } else if demo_mode {
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
                // Explicit session mode: watch status file directly
                let watch_path = initial_status_path.clone();
                let log_path = initial_status_path.clone();
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
            }

            Ok(())
        })
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if let Some(lock) = lock_for_cleanup.lock().unwrap().as_ref() {
                    let _ = fs::remove_file(lock);
                }
                // Don't delete status file — it belongs to the session, not the pet
            }
        })
        .run(tauri::generate_context!())
        .expect("error running app");
}
