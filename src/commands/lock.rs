use std::path::Path;
use crate::error::{AgentChatError, Result};
use crate::storage::{config, lockfile, paths};

pub fn acquire(root: &Path, glob: &str) -> Result<()> {
    let name = std::env::var("AGENT_CHAT_NAME")
        .map_err(|_| AgentChatError::MissingEnv("AGENT_CHAT_NAME".to_string()))?;
    let session_id = std::env::var("AGENT_CHAT_SESSION_ID")
        .map_err(|_| AgentChatError::MissingEnv("AGENT_CHAT_SESSION_ID".to_string()))?;

    let config = config::read_config(&paths::config_path(root))?;
    let locks_dir = paths::locks_dir(root);

    lockfile::acquire(&locks_dir, glob, &name, &session_id, config.lock_ttl_secs)?;
    println!("Locked: {}", glob);
    Ok(())
}

pub fn release(root: &Path, glob: &str) -> Result<()> {
    let session_id = std::env::var("AGENT_CHAT_SESSION_ID")
        .map_err(|_| AgentChatError::MissingEnv("AGENT_CHAT_SESSION_ID".to_string()))?;

    let locks_dir = paths::locks_dir(root);
    lockfile::release(&locks_dir, glob, &session_id)?;
    println!("Unlocked: {}", glob);
    Ok(())
}

pub fn list(root: &Path) -> Result<()> {
    let locks_dir = paths::locks_dir(root);
    let locks = lockfile::list_active(&locks_dir)?;

    if locks.is_empty() {
        println!("No active locks.");
        return Ok(());
    }

    println!("{:<30} {:<15} {}", "PATTERN", "OWNER", "TTL");
    for lock in &locks {
        let remaining = (lock.acquired_at + lock.ttl_secs).saturating_sub(
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        println!("{:<30} {:<15} {}s", lock.glob, lock.owner, remaining);
    }
    Ok(())
}
