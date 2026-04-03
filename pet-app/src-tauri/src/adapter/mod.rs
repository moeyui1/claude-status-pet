/// Adapter system: converts agent-specific hook data into normalized events.
///
/// Each adapter parses a specific agent's stdin JSON format and produces
/// a NormalizedEvent that the write-status command can process.

pub mod claude;
pub mod copilot;

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
    #[serde(default)]
    pub hook_event_name: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    // Copilot fields
    #[serde(alias = "toolName", default)]
    pub tool_name_copilot: Option<String>,
    #[serde(alias = "toolArgs", default)]
    pub tool_args: Option<String>,
    // Error info
    #[serde(default)]
    pub error: Option<serde_json::Value>,
}

pub trait Adapter {
    fn parse(&self, stdin: &StdinInput) -> Option<NormalizedEvent>;
}

/// Get adapter by name
pub fn get_adapter(name: &str) -> Option<Box<dyn Adapter>> {
    match name {
        "claude" => Some(Box::new(claude::ClaudeAdapter)),
        "copilot" => Some(Box::new(copilot::CopilotAdapter)),
        _ => None,
    }
}
