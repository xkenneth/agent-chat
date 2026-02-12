use std::path::Path;
use serde_json::json;
use crate::error::Result;
use crate::hooks::stdin;
use crate::storage::{lockfile, paths};

pub fn run(root: &Path) -> Result<()> {
    let session_id = match std::env::var("AGENT_CHAT_SESSION_ID") {
        Ok(id) => id,
        Err(_) => return Ok(()), // No session, can't check locks
    };

    let input = stdin::read_pre_tool_use()?;

    // Extract file_path from tool_input
    let file_path = match input.tool_input.get("file_path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return Ok(()), // No file path in input
    };

    let locks_dir = paths::locks_dir(root);
    if let Some(lock) = lockfile::check_file(&locks_dir, file_path, &session_id)? {
        // Output hookSpecificOutput JSON to warn the agent
        let warning = json!({
            "hookSpecificOutput": {
                "message": format!(
                    "WARNING: {} is locked by {} (pattern: {}). Coordinate before editing.",
                    file_path, lock.owner, lock.glob
                )
            }
        });
        print!("{}", serde_json::to_string(&warning)?);
    }
    // Silent when no lock conflict

    Ok(())
}
