/// GitHub Copilot adapter
///
/// Parses Copilot hook data:
/// - Event name: from COPILOT_HOOK_EVENT env var (camelCase)
/// - Tool names: snake_case (replace_string_in_file, read_file, etc.)
/// - Tool args: JSON string in toolArgs field (needs secondary parse)
/// - Session ID: not provided — generated from MD5 of cwd
///
/// Quirks handled:
/// - sessionStart: launch_only=true (don't write status, avoids racing with userPromptSubmitted)
/// - postToolUse: mapped to "prompt"/thinking (avoids idle flash between tools)
/// - sessionEnd: writes offline (does NOT close the window)

use super::{Adapter, NormalizedEvent, StdinInput};
use std::path::Path;

pub struct CopilotAdapter;

impl Adapter for CopilotAdapter {
    fn parse(&self, stdin: &StdinInput) -> Option<NormalizedEvent> {
        // Event name comes from --copilot-event CLI arg (Copilot CLI doesn't put it in stdin)
        // Fallback: stdin hookEventName (VS Code), then env var (legacy)
        let hook_from_arg = std::env::args().collect::<Vec<_>>()
            .windows(2)
            .find(|w| w[0] == "--copilot-event")
            .map(|w| w[1].clone());
        let hook_from_stdin = stdin.hook_event_name.as_deref().filter(|s| !s.is_empty()).map(|s| s.to_string());
        let hook_from_env = std::env::var("COPILOT_HOOK_EVENT").ok();
        let hook = hook_from_arg.or(hook_from_stdin).or(hook_from_env).unwrap_or_else(|| "unknown".into());
        let cwd = stdin.cwd.as_deref().unwrap_or("");
        // Use sessionId from stdin if available, otherwise hash from cwd
        let session_id = stdin.session_id.as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("copilot-{}", s))
            .unwrap_or_else(|| format!("copilot-{}", md5_short(if cwd.is_empty() { "default" } else { cwd })));
        let session_name = format!(
            "{} (Copilot)",
            Path::new(cwd).file_name().and_then(|n| n.to_str()).unwrap_or("copilot")
        );

        let tool_name = stdin.tool_name.as_deref().unwrap_or("");

        // Parse toolArgs (may be a JSON string or object)
        let tool_args: serde_json::Value = match &stdin.tool_args {
            Some(serde_json::Value::String(s)) => serde_json::from_str(s).unwrap_or(serde_json::Value::Null),
            Some(v) => v.clone(),
            None => stdin.tool_input.clone().unwrap_or(serde_json::Value::Null),
        };

        let (event, tool, detail, launch_only) = match hook.as_str() {
            "sessionStart" => {
                ("prompt".into(), String::new(), "Processing prompt...".into(), false)
            }
            "userPromptSubmitted" => {
                // Ignore — sessionStart already sets thinking state
                return None;
            }
            "preToolUse" => {
                let file = get_str(&tool_args, &["file", "filePath", "path"]);
                let command = get_str(&tool_args, &["command"]);
                let pattern = get_str(&tool_args, &["pattern", "query"]);
                let glob_pat = get_str(&tool_args, &["glob", "pattern"]);

                let detail = match crate::status_map::tool_to_state(tool_name) {
                    "editing" => format!("Editing {}", basename(&file.unwrap_or_default())),
                    "reading" => {
                        if tool_name.contains("fetch") {
                            "Fetching web page...".into()
                        } else if tool_name.contains("list") {
                            format!("Listing {}", basename(&file.unwrap_or_default()))
                        } else {
                            format!("Reading {}", basename(&file.unwrap_or_default()))
                        }
                    }
                    "searching" => {
                        let q = pattern.or(glob_pat).unwrap_or_default();
                        if tool_name.contains("find") || tool_name.contains("glob") || tool_name.contains("file_search") {
                            format!("Finding: {}", truncate(&q, 30))
                        } else {
                            format!("Searching: {}", truncate(&q, 30))
                        }
                    }
                    "running" => format!("Running: {}", truncate(&command.unwrap_or_default(), 40)),
                    _ => format!("Using {}", tool_name),
                };

                ("tool".into(), tool_name.to_string(), detail, false)
            }
            "postToolUse" => {
                // Quirk: map to thinking, not idle (avoids flash between tools)
                ("prompt".into(), String::new(), "Processing...".into(), false)
            }
            "stop" => {
                ("done".into(), String::new(), "Done".into(), false)
            }
            "errorOccurred" => {
                let msg = stdin.error.as_ref()
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                ("error".into(), String::new(), format!("Error: {}", truncate(msg, 40)), false)
            }
            "sessionEnd" => {
                let reason = stdin.reason.as_deref().unwrap_or("complete");
                match reason {
                    "complete" => ("done".into(), String::new(), "Done".into(), false),
                    "error" => ("error".into(), String::new(), "Session error".into(), false),
                    "abort" | "user_exit" => ("done".into(), String::new(), "Session closed".into(), false),
                    // "timeout" or unknown → offline (sleep animation)
                    _ => ("offline".into(), String::new(), "Session ended".into(), false),
                }
            }
            _ => {
                ("done".into(), String::new(), "Waiting for input".into(), false)
            }
        };

        Some(NormalizedEvent {
            event,
            tool,
            detail,
            session_id,
            session_name,
            launch_only,
        })
    }
}

fn get_str(v: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(s) = v.get(key).and_then(|v| v.as_str()) {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn basename(path: &str) -> &str {
    path.rsplit(&['/', '\\']).next().unwrap_or(path)
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { return s; }
    let mut idx = max;
    while idx > 0 && !s.is_char_boundary(idx) { idx -= 1; }
    &s[..idx]
}

/// Simple MD5-like hash (first 8 hex chars) for session ID generation
fn md5_short(input: &str) -> String {
    // Simple hash — not cryptographic, just for unique IDs
    let mut hash: u64 = 0;
    for (i, b) in input.bytes().enumerate() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u64).wrapping_add(i as u64);
    }
    format!("{:08x}", hash)
}
