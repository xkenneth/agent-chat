use std::path::Path;
use crate::error::{AgentChatError, Result};
use crate::hooks::stdin;
use crate::names;
use crate::storage::{paths, session};

pub fn run(root: &Path) -> Result<()> {
    let input = stdin::read_session_start()?;
    let session_id = &input.session_id;

    let sessions_dir = paths::sessions_dir(root);

    // Check if already registered (idempotent)
    let name = if let Some(existing) = session::read_session(&sessions_dir, session_id)? {
        existing
    } else {
        let name = names::generate_name();
        session::write_session(&sessions_dir, session_id, &name)?;
        name
    };

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

    println!("You are {}. Use 'agent-chat say <message>' to talk, 'agent-chat read' to check messages.", name);
    Ok(())
}
