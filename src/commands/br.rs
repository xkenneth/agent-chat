use std::process::Command;
use crate::error::{AgentChatError, Result};

/// Check that `br` is available on PATH. Returns a friendly error if not.
pub fn require_br_in_path() -> Result<()> {
    let output = Command::new("br")
        .arg("--version")
        .output()
        .map_err(|_| AgentChatError::Other(
            "br (beads_rust) not found in PATH. Install it first: cargo install beads_rust".to_string()
        ))?;

    if !output.status.success() {
        return Err(AgentChatError::Other(
            "br (beads_rust) not found in PATH. Install it first: cargo install beads_rust".to_string()
        ));
    }

    Ok(())
}

/// Get the title of a br issue by its ID.
pub fn get_issue_title(id: &str) -> Result<String> {
    let output = Command::new("br")
        .args(["show", id, "--json"])
        .output()
        .map_err(|e| AgentChatError::Other(format!("Failed to run br show: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AgentChatError::Other(format!("br show failed: {}", stderr.trim())));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let title = json["title"]
        .as_str()
        .unwrap_or("(untitled)")
        .to_string();

    Ok(title)
}
