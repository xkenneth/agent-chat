use std::path::Path;
use std::process::Command;
use crate::commands::{br, say};
use crate::error::{AgentChatError, Result};
use crate::storage::{focus, paths};

pub fn run(root: &Path, id: &str) -> Result<()> {
    br::require_br_in_path()?;

    let name = std::env::var("AGENT_CHAT_NAME")
        .map_err(|_| AgentChatError::MissingEnv("AGENT_CHAT_NAME".to_string()))?;

    let output = Command::new("br")
        .args(["update", id, "--status", "in_progress", "--assignee", &name])
        .output()
        .map_err(|e| AgentChatError::Other(format!("Failed to run br update: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AgentChatError::Other(format!("br update failed: {}", stderr.trim())));
    }

    let title = br::get_issue_title(id)?;

    // Check for focus overlaps (advisory warning only)
    let session_id = std::env::var("AGENT_CHAT_SESSION_ID").unwrap_or_default();
    let focuses_dir = paths::focuses_dir(root);
    if let Ok(overlaps) = focus::find_overlapping(&focuses_dir, &title, &session_id) {
        for o in &overlaps {
            eprintln!(
                "WARNING: {} is focused on '{}' — may overlap with bead {} '{}'",
                o.owner, o.focus, id, title
            );
        }
    }

    say::run(root, &format!("starting br-{}: {}", id, title))?;

    Ok(())
}
