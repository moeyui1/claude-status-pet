#[cfg(test)]
mod tests {
    use crate::adapter::{self, Adapter, StdinInput};
    use crate::status_map;

    // ── status_map tests ──

    #[test]
    fn test_event_to_state() {
        assert_eq!(status_map::event_to_state("prompt"), "thinking");
        assert_eq!(status_map::event_to_state("tool"), "running");
        assert_eq!(status_map::event_to_state("done"), "idle");
        assert_eq!(status_map::event_to_state("error"), "error");
        assert_eq!(status_map::event_to_state("offline"), "offline");
        assert_eq!(status_map::event_to_state("wait"), "waiting");
        assert_eq!(status_map::event_to_state("subagent"), "delegating");
        assert_eq!(status_map::event_to_state("unknown_event"), "thinking");
    }

    #[test]
    fn test_tool_to_state_claude_tools() {
        assert_eq!(status_map::tool_to_state("Edit"), "editing");
        assert_eq!(status_map::tool_to_state("Write"), "editing");
        assert_eq!(status_map::tool_to_state("NotebookEdit"), "editing");
        assert_eq!(status_map::tool_to_state("Read"), "reading");
        assert_eq!(status_map::tool_to_state("WebFetch"), "reading");
        assert_eq!(status_map::tool_to_state("Grep"), "searching");
        assert_eq!(status_map::tool_to_state("Glob"), "searching");
        assert_eq!(status_map::tool_to_state("WebSearch"), "searching");
        assert_eq!(status_map::tool_to_state("Bash"), "running");
        assert_eq!(status_map::tool_to_state("Agent"), "delegating");
        assert_eq!(status_map::tool_to_state("Skill"), "delegating");
    }

    #[test]
    fn test_tool_to_state_copilot_tools() {
        assert_eq!(status_map::tool_to_state("replace_string_in_file"), "editing");
        assert_eq!(status_map::tool_to_state("create_file"), "editing");
        assert_eq!(status_map::tool_to_state("edit_file"), "editing");
        assert_eq!(status_map::tool_to_state("read_file"), "reading");
        assert_eq!(status_map::tool_to_state("fetch_webpage"), "reading");
        assert_eq!(status_map::tool_to_state("list_dir"), "reading");
        assert_eq!(status_map::tool_to_state("grep_search"), "searching");
        assert_eq!(status_map::tool_to_state("semantic_search"), "searching");
        assert_eq!(status_map::tool_to_state("file_search"), "searching");
        assert_eq!(status_map::tool_to_state("run_in_terminal"), "running");
    }

    #[test]
    fn test_tool_to_state_mcp_tools() {
        assert_eq!(status_map::tool_to_state("mcp__github__search_code"), "searching");
        assert_eq!(status_map::tool_to_state("mcp__browser__fetch"), "reading");
        assert_eq!(status_map::tool_to_state("mcp__unknown__something"), "running");
    }

    #[test]
    fn test_tool_to_state_unknown_fallback() {
        assert_eq!(status_map::tool_to_state("some_random_tool"), "running");
        assert_eq!(status_map::tool_to_state(""), "running");
    }

    #[test]
    fn test_tool_detail_generation() {
        assert_eq!(status_map::tool_detail("Edit", "", "/path/to/main.rs"), "Editing main.rs");
        assert_eq!(status_map::tool_detail("Read", "", "/foo/bar.js"), "Reading bar.js");
        assert_eq!(status_map::tool_detail("Grep", "", "TODO"), "Searching: TODO");
        assert_eq!(status_map::tool_detail("Bash", "", ""), "Using Bash");
        // Explicit detail overrides
        assert_eq!(status_map::tool_detail("Edit", "Custom detail", "/foo"), "Custom detail");
    }

    // ── Claude adapter tests ──

    #[test]
    fn test_claude_prompt() {
        let stdin = make_stdin(Some("UserPromptSubmit"), None, None, Some("sess1"), Some("/proj"));
        let ev = adapter::claude::ClaudeAdapter.parse(&stdin).unwrap();
        assert_eq!(ev.event, "prompt");
        assert_eq!(ev.session_id, "sess1");
        assert_eq!(ev.session_name, "proj");
        assert!(!ev.launch_only);
    }

    #[test]
    fn test_claude_tool_edit() {
        let input = serde_json::json!({"file_path": "/foo/bar.rs"});
        let stdin = make_stdin(Some("PreToolUse"), Some("Edit"), Some(input), Some("s1"), Some("/proj"));
        let ev = adapter::claude::ClaudeAdapter.parse(&stdin).unwrap();
        assert_eq!(ev.event, "tool");
        assert_eq!(ev.tool, "Edit");
        assert!(ev.detail.contains("bar.rs"));
    }

    #[test]
    fn test_claude_tool_grep() {
        let input = serde_json::json!({"pattern": "TODO"});
        let stdin = make_stdin(Some("PreToolUse"), Some("Grep"), Some(input), Some("s1"), Some("/proj"));
        let ev = adapter::claude::ClaudeAdapter.parse(&stdin).unwrap();
        assert_eq!(ev.event, "tool");
        assert!(ev.detail.contains("TODO"));
    }

    #[test]
    fn test_claude_tool_mcp() {
        let stdin = make_stdin(Some("PreToolUse"), Some("mcp__github__search_code"), None, Some("s1"), Some("/proj"));
        let ev = adapter::claude::ClaudeAdapter.parse(&stdin).unwrap();
        assert!(ev.detail.contains("github: search_code"));
    }

    #[test]
    fn test_claude_stop() {
        let stdin = make_stdin(Some("Stop"), None, None, Some("s1"), Some("/proj"));
        let ev = adapter::claude::ClaudeAdapter.parse(&stdin).unwrap();
        assert_eq!(ev.event, "done");
    }

    #[test]
    fn test_claude_error() {
        let stdin = make_stdin(Some("StopFailure"), None, None, Some("s1"), Some("/proj"));
        let ev = adapter::claude::ClaudeAdapter.parse(&stdin).unwrap();
        assert_eq!(ev.event, "error");
    }

    #[test]
    fn test_claude_session_end() {
        let stdin = make_stdin(Some("SessionEnd"), None, None, Some("s1"), Some("/proj"));
        let ev = adapter::claude::ClaudeAdapter.parse(&stdin).unwrap();
        assert_eq!(ev.event, "offline");
    }

    #[test]
    fn test_claude_subagent() {
        let stdin = make_stdin(Some("SubagentStart"), None, None, Some("s1"), Some("/proj"));
        let ev = adapter::claude::ClaudeAdapter.parse(&stdin).unwrap();
        assert_eq!(ev.event, "subagent");
    }

    // ── Copilot adapter tests ──

    #[test]
    fn test_copilot_session_start_writes_thinking() {
        std::env::set_var("COPILOT_HOOK_EVENT", "sessionStart");
        let stdin = make_stdin(None, None, None, None, Some("/proj"));
        let ev = adapter::copilot::CopilotAdapter.parse(&stdin).unwrap();
        assert_eq!(ev.event, "prompt");
        assert!(!ev.launch_only);
        assert!(ev.session_id.starts_with("copilot-"));
    }

    #[test]
    fn test_copilot_prompt_ignored() {
        std::env::set_var("COPILOT_HOOK_EVENT", "userPromptSubmitted");
        let stdin = make_stdin(None, None, None, None, Some("/proj"));
        let ev = adapter::copilot::CopilotAdapter.parse(&stdin);
        assert!(ev.is_none());
    }

    #[test]
    fn test_copilot_post_tool_is_thinking() {
        std::env::set_var("COPILOT_HOOK_EVENT", "postToolUse");
        let stdin = make_stdin(None, None, None, None, Some("/proj"));
        let ev = adapter::copilot::CopilotAdapter.parse(&stdin).unwrap();
        assert_eq!(ev.event, "prompt"); // NOT done/idle
    }

    #[test]
    fn test_copilot_stop() {
        std::env::set_var("COPILOT_HOOK_EVENT", "stop");
        let stdin = make_stdin(None, None, None, None, Some("/proj"));
        let ev = adapter::copilot::CopilotAdapter.parse(&stdin).unwrap();
        assert_eq!(ev.event, "done");
    }

    #[test]
    fn test_copilot_session_end_complete_is_idle() {
        std::env::set_var("COPILOT_HOOK_EVENT", "sessionEnd");
        let stdin = make_stdin(None, None, None, None, Some("/proj"));
        let ev = adapter::copilot::CopilotAdapter.parse(&stdin).unwrap();
        assert_eq!(ev.event, "done"); // complete → idle
    }

    #[test]
    fn test_copilot_session_name_has_suffix() {
        std::env::set_var("COPILOT_HOOK_EVENT", "sessionStart");
        let stdin = make_stdin(None, None, None, None, Some("/projects/my-app"));
        let ev = adapter::copilot::CopilotAdapter.parse(&stdin).unwrap();
        assert!(ev.session_name.contains("Copilot"));
        assert!(ev.session_name.contains("my-app"));
    }

    // ── Helper ──

    fn make_stdin(
        hook: Option<&str>,
        tool: Option<&str>,
        tool_input: Option<serde_json::Value>,
        session_id: Option<&str>,
        cwd: Option<&str>,
    ) -> StdinInput {
        StdinInput {
            hook_event_name: hook.map(|s| s.to_string()),
            tool_name: tool.map(|s| s.to_string()),
            tool_input,
            session_id: session_id.map(|s| s.to_string()),
            cwd: cwd.map(|s| s.to_string()),
            tool_args: None,
            error: None,
            reason: None,
        }
    }
}
