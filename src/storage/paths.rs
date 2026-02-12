use std::path::{Path, PathBuf};
use crate::error::{AgentChatError, Result};

const DIR_NAME: &str = ".agent-chat";

/// Walk up from `start` to find the `.agent-chat/` directory.
/// Returns the path to `.agent-chat/` or an error if not found.
pub fn find_root(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join(DIR_NAME);
        if candidate.is_dir() {
            return Ok(candidate);
        }
        if !current.pop() {
            return Err(AgentChatError::NotInitialized);
        }
    }
}

/// Create the `.agent-chat/` directory structure at the given project root.
pub fn create_dirs(project_root: &Path) -> Result<()> {
    let base = project_root.join(DIR_NAME);
    std::fs::create_dir_all(base.join("log"))?;
    std::fs::create_dir_all(base.join("locks"))?;
    std::fs::create_dir_all(base.join("cursors"))?;
    std::fs::create_dir_all(base.join("sessions"))?;
    Ok(())
}

pub fn log_dir(root: &Path) -> PathBuf {
    root.join("log")
}

pub fn locks_dir(root: &Path) -> PathBuf {
    root.join("locks")
}

pub fn cursors_dir(root: &Path) -> PathBuf {
    root.join("cursors")
}

pub fn sessions_dir(root: &Path) -> PathBuf {
    root.join("sessions")
}

pub fn config_path(root: &Path) -> PathBuf {
    root.join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn find_root_discovers_agent_chat_dir() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".agent-chat");
        std::fs::create_dir(&base).unwrap();
        let nested = tmp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&nested).unwrap();

        let found = find_root(&nested).unwrap();
        assert_eq!(found, base);
    }

    #[test]
    fn find_root_returns_error_when_missing() {
        let tmp = TempDir::new().unwrap();
        let result = find_root(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn create_dirs_makes_all_subdirs() {
        let tmp = TempDir::new().unwrap();
        create_dirs(tmp.path()).unwrap();
        assert!(tmp.path().join(".agent-chat/log").is_dir());
        assert!(tmp.path().join(".agent-chat/locks").is_dir());
        assert!(tmp.path().join(".agent-chat/cursors").is_dir());
        assert!(tmp.path().join(".agent-chat/sessions").is_dir());
    }
}
