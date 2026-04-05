/// Adapter system: converts agent-specific hook data into normalized events.
///
/// Each adapter parses a specific agent's stdin JSON format and produces
/// a NormalizedEvent that the write-status command can process.

pub mod claude;
pub mod copilot;
pub mod vscode;

use serde::Deserialize;

/// Normalized event output from any adapter
pub struct NormalizedEvent {
    /// Generic event: prompt, tool, done, error, offline, wait, subagent
    pub event: String,
    /// Tool name (raw, will be mapped by status_map::tool_to_state)
    pub tool: String,
    /// Human-readable detail text
    pub detail: String,
    /// Session identifier
    pub session_id: String,
    /// Display name for the session
    pub session_name: String,
    /// If true, only launch GUI — don't write status file
    pub launch_only: bool,
}

/// Raw JSON from stdin (loosely typed for all adapters)
#[derive(Deserialize, Default)]
pub struct StdinInput {
    #[serde(alias = "hookEventName", default)]
    pub hook_event_name: Option<String>,
    #[serde(alias = "toolName", default)]
    pub tool_name: Option<String>,
    #[serde(alias = "toolInput", default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(alias = "sessionId", default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    // Copilot toolArgs (may be JSON string or object depending on event)
    #[serde(alias = "toolArgs", default)]
    pub tool_args: Option<serde_json::Value>,
    // Error info
    #[serde(default)]
    pub error: Option<serde_json::Value>,
    // Session end reason (complete, error, abort, timeout, user_exit)
    #[serde(default)]
    pub reason: Option<String>,
}

pub trait Adapter {
    fn parse(&self, stdin: &StdinInput) -> Option<NormalizedEvent>;
}

/// Get adapter by name
pub fn get_adapter(name: &str) -> Option<Box<dyn Adapter>> {
    match name {
        "claude" => Some(Box::new(claude::ClaudeAdapter)),
        "copilot" => Some(Box::new(copilot::CopilotAdapter)),
        "vscode" => Some(Box::new(vscode::VscodeAdapter)),
        _ => None,
    }
}

// ── Shared helpers used by multiple adapters ──

/// Extract the file basename from a path (e.g. "src/main.rs" → "main.rs")
pub fn basename(path: &str) -> &str {
    path.rsplit(&['/', '\\']).next().unwrap_or(path)
}

/// Truncate a string to `max` bytes, respecting UTF-8 char boundaries
pub fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { return s; }
    let mut idx = max;
    while idx > 0 && !s.is_char_boundary(idx) { idx -= 1; }
    &s[..idx]
}

/// Simple hash (first 8 hex chars) for session ID generation from cwd
pub fn md5_short(input: &str) -> String {
    let mut hash: u64 = 0;
    for (i, b) in input.bytes().enumerate() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u64).wrapping_add(i as u64);
    }
    format!("{:08x}", hash)
}

/// Extract a string value from a JSON object by key
pub fn extract_str(input: Option<&serde_json::Value>, key: &str) -> Option<String> {
    input?.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Extract a string value from a JSON object, trying multiple keys in order
pub fn get_str(v: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(s) = v.get(key).and_then(|v| v.as_str()) {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}
