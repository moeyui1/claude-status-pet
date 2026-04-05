/// VS Code Copilot adapter
///
/// Parses VS Code Copilot hook data:
/// - Events: PascalCase from stdin hookEventName (SessionStart, PreToolUse, Stop, etc.)
/// - Tool names: snake_case (replace_string_in_file, read_file, etc.) or camelCase (editFiles)
/// - Tool input: tool_input object in stdin
/// - Session ID: provided in stdin sessionId
///
/// Key differences from Copilot CLI adapter:
/// - Events come from stdin hookEventName (PascalCase), not CLI arg (camelCase)
/// - PostToolUse → thinking (prevents idle flash between tools)
/// - PreCompact → ignored (no status change)
/// - No sessionEnd/errorOccurred — Stop is the session end
/// - SubagentStart/SubagentStop supported

use super::{Adapter, NormalizedEvent, StdinInput, basename, truncate, md5_short, extract_str};
use std::path::Path;

pub struct VscodeAdapter;

impl Adapter for VscodeAdapter {
    fn parse(&self, stdin: &StdinInput) -> Option<NormalizedEvent> {
        let hook = stdin.hook_event_name.as_deref().unwrap_or("unknown");
        let tool_name = stdin.tool_name.as_deref().unwrap_or("");
        let tool_input = stdin.tool_input.as_ref();
        let cwd = stdin.cwd.as_deref().unwrap_or("");

        // VS Code provides sessionId in stdin
        let session_id = stdin.session_id.as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("vscode-{}", s))
            .unwrap_or_else(|| format!("vscode-{}", md5_short(if cwd.is_empty() { "default" } else { cwd })));

        let session_name = format!(
            "{} (VS Code)",
            Path::new(cwd).file_name().and_then(|n| n.to_str()).unwrap_or("vscode")
        );

        let (event, tool, detail) = match hook {
            "SessionStart" => {
                ("prompt".into(), String::new(), "Session started".into())
            }
            "UserPromptSubmit" => {
                ("prompt".into(), String::new(), "Processing your prompt...".into())
            }
            "PreToolUse" => {
                let file = extract_str(tool_input, "file_path")
                    .or_else(|| extract_str(tool_input, "filePath"))
                    .or_else(|| extract_str(tool_input, "path"));
                let command = extract_str(tool_input, "command");
                let pattern = extract_str(tool_input, "pattern")
                    .or_else(|| extract_str(tool_input, "query"));

                let detail = if tool_name.starts_with("mcp__") || tool_name.starts_with("mcp_") {
                    // MCP tools: format as "server: tool"
                    let sep = if tool_name.contains("__") { "__" } else { "_" };
                    let parts: Vec<&str> = tool_name.splitn(3, sep).collect();
                    if parts.len() >= 3 {
                        format!("{}: {}", parts[1], parts[2])
                    } else {
                        format!("Using {}", tool_name)
                    }
                } else {
                    match crate::status_map::tool_to_state(tool_name) {
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
                            let q = pattern.unwrap_or_default();
                            if tool_name.contains("file_search") || tool_name.contains("find") || tool_name.contains("glob") {
                                format!("Finding: {}", truncate(&q, 30))
                            } else {
                                format!("Searching: {}", truncate(&q, 30))
                            }
                        }
                        "running" => format!("Running: {}", truncate(&command.unwrap_or_default(), 40)),
                        "delegating" => "Delegating...".into(),
                        _ => format!("Using {}", tool_name),
                    }
                };

                ("tool".into(), tool_name.to_string(), detail)
            }
            "PostToolUse" => {
                // Map to thinking — prevents idle flash between consecutive tools
                ("prompt".into(), String::new(), "Processing...".into())
            }
            "PreCompact" => {
                // Context compaction — not status-relevant
                return None;
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
