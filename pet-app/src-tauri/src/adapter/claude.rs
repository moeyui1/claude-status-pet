/// Claude Code adapter
///
/// Parses Claude Code hook JSON from stdin:
/// - Events: PascalCase (UserPromptSubmit, PreToolUse, Stop, etc.)
/// - Tool names: PascalCase (Edit, Read, Bash, Grep, etc.)
/// - Tool input: snake_case keys (file_path, command, etc.)
/// - Session ID: provided in JSON

use super::{Adapter, NormalizedEvent, StdinInput, basename, truncate, extract_str};
use std::path::Path;

pub struct ClaudeAdapter;

impl Adapter for ClaudeAdapter {
    fn parse(&self, stdin: &StdinInput) -> Option<NormalizedEvent> {
        let hook = stdin.hook_event_name.as_deref().unwrap_or("unknown");
        let tool_name = stdin.tool_name.as_deref().unwrap_or("");
        let tool_input = stdin.tool_input.as_ref();
        let cwd = stdin.cwd.as_deref().unwrap_or("");
        let session_id = stdin.session_id.as_deref().unwrap_or("unknown");
        let session_name = Path::new(cwd)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(session_id);

        let (event, tool, detail) = match hook {
            "UserPromptSubmit" => {
                ("prompt".into(), String::new(), "Processing your prompt...".into())
            }
            "PreToolUse" => {
                let file = extract_file(tool_input);
                let command = extract_str(tool_input, "command");
                let pattern = extract_str(tool_input, "pattern")
                    .or_else(|| extract_str(tool_input, "query"));
                let description = extract_str(tool_input, "description")
                    .or_else(|| extract_str(tool_input, "skill"));

                let detail = if tool_name.starts_with("mcp__") {
                    // MCP tools: format as "server: tool"
                    let parts: Vec<&str> = tool_name.splitn(3, "__").collect();
                    if parts.len() >= 3 {
                        format!("{}: {}", parts[1], parts[2])
                    } else {
                        format!("Using {}", tool_name)
                    }
                } else {
                    match crate::status_map::tool_to_state(tool_name) {
                        "editing" => format!("Editing {}", basename(&file.unwrap_or_default())),
                        "reading" => format!("Reading {}", basename(&file.unwrap_or_default())),
                        "searching" => format!("Searching: {}", truncate(&pattern.unwrap_or_default(), 30)),
                        "running" => format!("Running: {}", truncate(&command.unwrap_or_default(), 40)),
                        "delegating" => description.unwrap_or_else(|| "Delegating...".into()),
                        _ => format!("Using {}", tool_name),
                    }
                };

                ("tool".into(), tool_name.to_string(), detail)
            }
            "SubagentStart" => {
                ("subagent".into(), "agent".into(), "Spawning sub-agent...".into())
            }
            "SubagentStop" => {
                ("prompt".into(), String::new(), "Sub-agent finished".into())
            }
            "Notification" => {
                ("wait".into(), String::new(), "Waiting for approval...".into())
            }
            "Stop" => {
                ("done".into(), String::new(), "Waiting for input".into())
            }
            "StopFailure" => {
                ("error".into(), String::new(), "Something went wrong".into())
            }
            "SessionEnd" => {
                ("offline".into(), String::new(), "Session ended".into())
            }
            "SessionStart" => {
                // Write initial idle status
                ("done".into(), String::new(), "Session started".into())
            }
            _ => {
                ("prompt".into(), String::new(), format!("{}", hook))
            }
        };

        Some(NormalizedEvent {
            event,
            tool,
            detail,
            session_id: session_id.to_string(),
            session_name: session_name.to_string(),
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
