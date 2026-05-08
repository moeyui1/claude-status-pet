/// GitHub Copilot adapter
///
/// Parses Copilot CLI hook data per the official reference:
/// https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-hooks-reference
///
/// - Event name: from `--copilot-event` CLI arg (camelCase, e.g. `preToolUse`)
/// - Tool names: standard Copilot CLI names (`bash`, `edit`, `view`, `grep`, `glob`,
///   `create`, `web_fetch`, `task`, `powershell`, `ask_user`)
/// - Tool args: `toolArgs` field — JSON string for `preToolUse`, object for `postToolUse`
/// - Session ID: `sessionId` from stdin (fallback: hash of cwd)
///
/// Quirks handled:
/// - postToolUse: mapped to "prompt"/thinking (avoids idle flash between tools)
/// - postToolUseFailure: mapped to "error" with the error message
/// - preCompact / non-actionable notifications: ignored (no status change)
/// - notification (permission_prompt / elicitation_dialog): mapped to "waiting"
/// - permissionRequest: mapped to "waiting"
/// - subagentStart / subagentStop: mapped to "delegating" / "thinking"
/// - sessionEnd: writes offline/idle/error per `reason` (does NOT close the window)

use super::{Adapter, NormalizedEvent, StdinInput, basename, truncate, md5_short, get_str};
use std::path::Path;

pub struct CopilotAdapter;

impl Adapter for CopilotAdapter {
    fn parse(&self, stdin: &StdinInput) -> Option<NormalizedEvent> {
        // Event name comes from --copilot-event CLI arg (Copilot CLI doesn't put it in stdin)
        // Fallback: stdin hookEventName, then env var (legacy)
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
                ("prompt".into(), String::new(), "Processing your prompt...".into(), false)
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
            "postToolUseFailure" => {
                // Tool failed — surface as error with the message
                let msg = stdin.error.as_ref()
                    .and_then(error_message)
                    .unwrap_or_else(|| format!("{} failed", tool_name));
                ("error".into(), String::new(), format!("Error: {}", truncate(&msg, 40)), false)
            }
            "stop" | "agentStop" => {
                ("done".into(), String::new(), "Done".into(), false)
            }
            "subagentStart" => {
                let name = stdin.agent_name.as_deref().unwrap_or("sub-agent");
                ("subagent".into(), "agent".into(), format!("Spawning {}...", name), false)
            }
            "subagentStop" => {
                ("prompt".into(), String::new(), "Sub-agent finished".into(), false)
            }
            "preCompact" => {
                // Context compaction — not status-relevant
                return None;
            }
            "permissionRequest" => {
                // Permission prompt before tool runs
                ("wait".into(), String::new(), "Waiting for approval...".into(), false)
            }
            "notification" => {
                // Fire-and-forget notification. Only surface user-actionable types.
                match stdin.notification_type.as_deref().unwrap_or("") {
                    "permission_prompt" => {
                        ("wait".into(), String::new(), "Waiting for approval...".into(), false)
                    }
                    "elicitation_dialog" => {
                        ("wait".into(), String::new(), "Waiting for input...".into(), false)
                    }
                    // shell_completed, agent_completed, agent_idle, etc. — ignore
                    _ => return None,
                }
            }
            "errorOccurred" => {
                let msg = stdin.error.as_ref()
                    .and_then(error_message)
                    .unwrap_or_else(|| "Unknown error".to_string());
                ("error".into(), String::new(), format!("Error: {}", truncate(&msg, 40)), false)
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

/// Extract a human-readable message from an `error` JSON value.
/// Handles both:
/// - postToolUseFailure: `error: string`
/// - errorOccurred: `error: { message: string, ... }`
fn error_message(value: &serde_json::Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        return Some(s.to_string());
    }
    value.get("message")
        .and_then(|m| m.as_str())
        .map(|s| s.to_string())
}
