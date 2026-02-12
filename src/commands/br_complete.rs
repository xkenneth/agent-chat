use std::path::Path;
use std::process::Command;
use crate::commands::{br, say};
use crate::error::{AgentChatError, Result};

pub fn run(root: &Path, id: &str, reason: Option<&str>) -> Result<()> {
    br::require_br_in_path()?;

    // Get title before closing
    let title = br::get_issue_title(id)?;

    let mut cmd = Command::new("br");
    cmd.args(["close", id]);
    if let Some(r) = reason {
        cmd.args(["--reason", r]);
    }

    let output = cmd.output()
        .map_err(|e| AgentChatError::Other(format!("Failed to run br close: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AgentChatError::Other(format!("br close failed: {}", stderr.trim())));
    }

    say::run(root, &format!("completed br-{}: {}", id, title))?;

    Ok(())
}
