/// Universal tool name → pet state mapping.
/// Used by all adapters and CLI args mode.

/// All possible pet states
pub const STATES: &[&str] = &[
    "idle", "thinking", "reading", "editing", "searching",
    "running", "delegating", "waiting", "error", "offline",
];

/// Map a generic event name to a pet state
pub fn event_to_state(event: &str) -> &'static str {
    match event {
        "prompt" => "thinking",
        "tool" => "running", // refined by tool_to_state
        "done" => "idle",
        "error" => "error",
        "offline" => "offline",
        "closed" => "closed",
        "wait" => "waiting",
        "subagent" => "delegating",
        _ => "thinking",
    }
}

/// Map a tool name to a pet state using fuzzy keyword matching.
/// Works with any agent's tool names (Edit, replace_string_in_file, etc.)
pub fn tool_to_state(tool: &str) -> &'static str {
    let t = tool.to_lowercase();

    if contains_any(&t, &["edit", "write", "replace", "create_file", "notebook"]) {
        "editing"
    } else if contains_any(&t, &["read", "view", "fetch", "list_dir", "web_fetch"]) {
        "reading"
    } else if contains_any(&t, &["grep", "search", "find", "glob"]) {
        "searching"
    } else if contains_any(&t, &["bash", "terminal", "run", "shell", "powershell"]) {
        "running"
    } else if contains_any(&t, &["agent", "skill", "delegate", "subagent", "task"]) {
        "delegating"
    } else {
        "running" // fallback for unknown tools
    }
}

/// Generate a human-readable detail string from tool name and context
pub fn tool_detail(tool: &str, detail: &str, file: &str) -> String {
    if !detail.is_empty() {
        return detail.to_string();
    }

    let state = tool_to_state(tool);
    let basename = file.rsplit(&['/', '\\']).next().unwrap_or(file);

    match state {
        "editing" => format!("Editing {}", basename),
        "reading" => format!("Reading {}", basename),
        "searching" => {
            if file.is_empty() {
                format!("Searching...")
            } else {
                format!("Searching: {}", truncate(file, 30))
            }
        }
        "running" => {
            if file.is_empty() {
                format!("Using {}", tool)
            } else {
                format!("Running: {}", truncate(file, 40))
            }
        }
        "delegating" => format!("Delegating..."),
        _ => format!("Using {}", tool),
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_tools() {
        assert_eq!(tool_to_state("Edit"), "editing");
        assert_eq!(tool_to_state("Write"), "editing");
        assert_eq!(tool_to_state("Read"), "reading");
        assert_eq!(tool_to_state("WebFetch"), "reading");
        assert_eq!(tool_to_state("Grep"), "searching");
        assert_eq!(tool_to_state("Glob"), "searching");
        assert_eq!(tool_to_state("Bash"), "running");
        assert_eq!(tool_to_state("Agent"), "delegating");
        assert_eq!(tool_to_state("Skill"), "delegating");
    }

    #[test]
    fn test_copilot_tools() {
        assert_eq!(tool_to_state("replace_string_in_file"), "editing");
        assert_eq!(tool_to_state("create_file"), "editing");
        assert_eq!(tool_to_state("read_file"), "reading");
        assert_eq!(tool_to_state("fetch_webpage"), "reading");
        assert_eq!(tool_to_state("list_dir"), "reading");
        assert_eq!(tool_to_state("grep_search"), "searching");
        assert_eq!(tool_to_state("semantic_search"), "searching");
        assert_eq!(tool_to_state("file_search"), "searching");
        assert_eq!(tool_to_state("run_in_terminal"), "running");
    }

    #[test]
    fn test_mcp_tools() {
        assert_eq!(tool_to_state("mcp__github__search_code"), "searching");
        assert_eq!(tool_to_state("mcp__browser__fetch"), "reading");
        assert_eq!(tool_to_state("mcp__unknown__something"), "running");
    }

    #[test]
    fn test_events() {
        assert_eq!(event_to_state("prompt"), "thinking");
        assert_eq!(event_to_state("tool"), "running");
        assert_eq!(event_to_state("done"), "idle");
        assert_eq!(event_to_state("error"), "error");
        assert_eq!(event_to_state("offline"), "offline");
    }
}
