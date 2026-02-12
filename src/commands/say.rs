use std::path::Path;
use crate::error::{AgentChatError, Result};
use crate::storage::{log, paths};

pub fn run(root: &Path, message: &str) -> Result<()> {
    let name = std::env::var("AGENT_CHAT_NAME")
        .map_err(|_| AgentChatError::MissingEnv("AGENT_CHAT_NAME".to_string()))?;

    let log_dir = paths::log_dir(root);
    log::write_message(&log_dir, &name, message)?;
    Ok(())
}
