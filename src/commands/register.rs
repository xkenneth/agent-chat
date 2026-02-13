use std::path::Path;
use serde_json::json;
use crate::error::{AgentChatError, Result};
use crate::format;
use crate::hooks::stdin;
use crate::names;
use crate::storage::{cursor, focus, log, paths, session};

pub fn run(root: &Path, session_id: Option<&str>) -> Result<()> {
    let session_id = resolve_session_id(session_id)?;

    let sessions_dir = paths::sessions_dir(root);
    let log_dir = paths::log_dir(root);
    let cursors_dir = paths::cursors_dir(root);
    let cursor_file = cursor::cursor_path(&cursors_dir, &session_id);

    // Check if already registered (idempotent)
    let (name, is_new) = if let Some(existing) = session::read_session(&sessions_dir, &session_id)? {
        (existing, false)
    } else {
        let name = names::generate_name();
        session::write_session(&sessions_dir, &session_id, &name)?;
        (name, true)
    };

    // Post join message for new sessions only
    if is_new {
        log::write_message(&log_dir, &name, "joined the chat")?;
    }

    // Write to CLAUDE_ENV_FILE if set
    if let Ok(env_file) = std::env::var("CLAUDE_ENV_FILE") {
        let content = format!(
            "export AGENT_CHAT_NAME={}\nexport AGENT_CHAT_SESSION_ID={}\n",
            name, session_id
        );
        std::fs::write(&env_file, content).map_err(|e| {
            AgentChatError::Other(format!("Failed to write CLAUDE_ENV_FILE: {}", e))
        })?;
    }

    // Build identity string
    let mut identity = format!(
        "You are {}. Use 'agent-chat say <message>' to talk, 'agent-chat read' to check messages.",
        name
    );

    // Append active focuses from other agents
    let focuses_dir = paths::focuses_dir(root);
    if let Ok(focuses) = focus::list_active(&focuses_dir) {
        let other_focuses: Vec<_> = focuses.iter().filter(|f| f.owner != name).collect();
        if !other_focuses.is_empty() {
            identity.push_str("\n\n[Active agent focuses]");
            for f in &other_focuses {
                identity.push_str(&format!("\n  - {} is focused on: {}", f.owner, f.focus));
            }
        }
    }

    // Inject existing unread messages
    let unread = cursor::get_unread_messages(&log_dir, &cursor_file, 50, Some(&name))?;
    let context = if !unread.is_empty() {
        let formatted = format::format_messages_from_paths(&unread);
        cursor::advance(&cursor_file)?;
        format!("{}\n{}", identity, formatted)
    } else {
        // Still advance cursor so we don't re-deliver our own join message later
        if is_new {
            cursor::advance(&cursor_file)?;
        }
        identity
    };

    let output = json!({
        "hookSpecificOutput": {
            "additionalContext": context
        }
    });
    print!("{}", output);
    Ok(())
}

fn resolve_session_id(explicit: Option<&str>) -> Result<String> {
    if let Some(id) = explicit {
        let trimmed = id.trim();
        if trimmed.is_empty() {
            return Err(AgentChatError::Other(
                "session_id cannot be empty".to_string()
            ));
        }
        return Ok(trimmed.to_string());
    }

    let input = stdin::read_session_start()?;
    Ok(input.session_id)
}
