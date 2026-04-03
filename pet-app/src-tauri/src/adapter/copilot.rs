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
        let hook = std::env::var("COPILOT_HOOK_EVENT").unwrap_or_else(|_| "unknown".into());
        let cwd = stdin.cwd.as_deref().unwrap_or("");
        let session_id = format!("copilot-{}", md5_short(if cwd.is_empty() { "default" } else { cwd }));
        let session_name = format!(
            "{} (Copilot)",
            Path::new(cwd).file_name().and_then(|n| n.to_str()).unwrap_or("copilot")
        );

        let tool_name = stdin.tool_name_copilot.as_deref()
            .or(stdin.tool_name.as_deref())
            .unwrap_or("");

        // Parse toolArgs (may be a JSON string)
        let tool_args: serde_json::Value = stdin.tool_args.as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .or_else(|| stdin.tool_input.clone())
            .unwrap_or(serde_json::Value::Null);

        let (event, tool, detail, launch_only) = match hook.as_str() {
            "sessionStart" => {
                // Quirk: don't write status — races with userPromptSubmitted
                ("done".into(), String::new(), "Session started".into(), true)
            }
            "userPromptSubmitted" => {
                ("prompt".into(), String::new(), "Processing prompt...".into(), false)
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
                // Quirk: write offline, don't close window
                ("offline".into(), String::new(), "Session ended".into(), false)
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
    if s.len() <= max { s } else { &s[..max] }
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
