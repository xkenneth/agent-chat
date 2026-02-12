use std::path::Path;
use std::process::Command;
use crate::commands::{br, say};
use crate::error::{AgentChatError, Result};

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
    say::run(root, &format!("starting br-{}: {}", id, title))?;

    Ok(())
}
