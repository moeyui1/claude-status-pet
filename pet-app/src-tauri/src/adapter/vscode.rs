/// VS Code Copilot adapter
///
/// Parses VS Code Copilot hook data from stdin:
/// - Events: PascalCase in stdin `hookEventName` (PreToolUse, PostToolUse, etc.)
/// - Tool names: snake_case `tool_name` (edit_file, read_file, etc.)
/// - Tool input: object `tool_input` (not a JSON string)
/// - Session ID: provided in stdin `sessionId`
/// - CWD: provided in stdin `cwd`
///
/// Key differences from Copilot CLI:
/// - hookEventName is in stdin JSON (not passed via CLI arg)
/// - sessionId is in stdin (no need to hash from cwd)
/// - tool_input is a JSON object (not a JSON string in toolArgs)
/// - Uses PascalCase event names (same as Claude Code)

use super::{Adapter, NormalizedEvent, StdinInput};
use std::path::Path;

pub struct VscodeAdapter;

impl Adapter for VscodeAdapter {
    fn parse(&self, stdin: &StdinInput) -> Option<NormalizedEvent> {
        let hook = stdin.hook_event_name.as_deref().unwrap_or("unknown");
        let cwd = stdin.cwd.as_deref().unwrap_or("");
        let session_id = stdin.session_id.as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("vscode-{}", s))
            .unwrap_or_else(|| format!("vscode-{}", md5_short(if cwd.is_empty() { "default" } else { cwd })));
        let session_name = format!(
            "{} (VS Code)",
            Path::new(cwd).file_name().and_then(|n| n.to_str()).unwrap_or("copilot")
        );

        let tool_name = stdin.tool_name.as_deref().unwrap_or("");
        let tool_input = stdin.tool_input.as_ref();

        let (event, tool, detail) = match hook {
            "SessionStart" => {
                ("prompt".into(), String::new(), "Processing prompt...".into())
            }
            "UserPromptSubmit" => {
                // Ignore — SessionStart already sets thinking
                return None;
            }
            "PreToolUse" => {
                let file = extract_file(tool_input);
                let command = extract_str(tool_input, "command");
                let pattern = extract_str(tool_input, "pattern")
                    .or_else(|| extract_str(tool_input, "query"));

                let detail = match crate::status_map::tool_to_state(tool_name) {
                    "editing" => format!("Editing {}", basename(&file.unwrap_or_default())),
                    "reading" => format!("Reading {}", basename(&file.unwrap_or_default())),
                    "searching" => format!("Searching: {}", truncate(&pattern.unwrap_or_default(), 30)),
                    "running" => format!("Running: {}", truncate(&command.unwrap_or_default(), 40)),
                    "delegating" => "Delegating...".into(),
                    _ => format!("Using {}", tool_name),
                };

                ("tool".into(), tool_name.to_string(), detail)
            }
            "PostToolUse" => {
                // Map to thinking (avoids idle flash between tools)
                ("prompt".into(), String::new(), "Processing...".into())
            }
            "SubagentStart" => {
                ("subagent".into(), "agent".into(), "Spawning sub-agent...".into())
            }
            "SubagentStop" => {
                ("prompt".into(), String::new(), "Sub-agent finished".into())
            }
            "Stop" => {
                ("done".into(), String::new(), "Done".into())
            }
            _ => {
                ("prompt".into(), String::new(), format!("{}", hook))
            }
        };

        Some(NormalizedEvent {
            event,
            tool,
            detail,
            session_id,
            session_name,
            launch_only: false,
        })
    }
}

fn extract_file(input: Option<&serde_json::Value>) -> Option<String> {
    let v = input?;
    v.get("file_path")
        .or_else(|| v.get("path"))
        .or_else(|| v.get("file"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn extract_str(input: Option<&serde_json::Value>, key: &str) -> Option<String> {
    input?.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
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

fn md5_short(input: &str) -> String {
    let mut hash: u64 = 0;
    for (i, b) in input.bytes().enumerate() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u64).wrapping_add(i as u64);
    }
    format!("{:08x}", hash)
}
