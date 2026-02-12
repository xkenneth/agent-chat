use std::path::{Path, PathBuf};
use std::fs;
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

/// Return the user's home directory from `$HOME`.
pub fn home_dir() -> Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| AgentChatError::MissingEnv("HOME".into()))
}

/// Append `pattern` to `.git/info/exclude` if not already present.
/// No-ops silently if the project is not a git repo.
pub fn add_git_exclude(project_root: &Path, pattern: &str) -> Result<()> {
    let git_dir = project_root.join(".git");
    if !git_dir.is_dir() {
        return Ok(());
    }
    let info_dir = git_dir.join("info");
    fs::create_dir_all(&info_dir)?;
    let exclude_path = info_dir.join("exclude");

    let existing = if exclude_path.exists() {
        fs::read_to_string(&exclude_path)?
    } else {
        String::new()
    };

    if existing.lines().any(|line| line.trim() == pattern) {
        return Ok(());
    }

    let mut content = existing;
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(pattern);
    content.push('\n');
    fs::write(&exclude_path, content)?;
    Ok(())
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

    #[test]
    fn add_git_exclude_appends_pattern() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".git/info")).unwrap();
        add_git_exclude(tmp.path(), ".agent-chat/").unwrap();
        let content = std::fs::read_to_string(tmp.path().join(".git/info/exclude")).unwrap();
        assert!(content.contains(".agent-chat/"));
    }

    #[test]
    fn add_git_exclude_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".git/info")).unwrap();
        add_git_exclude(tmp.path(), ".agent-chat/").unwrap();
        add_git_exclude(tmp.path(), ".agent-chat/").unwrap();
        let content = std::fs::read_to_string(tmp.path().join(".git/info/exclude")).unwrap();
        assert_eq!(content.matches(".agent-chat/").count(), 1);
    }

    #[test]
    fn add_git_exclude_noop_without_git() {
        let tmp = TempDir::new().unwrap();
        // No .git directory — should succeed silently
        add_git_exclude(tmp.path(), ".agent-chat/").unwrap();
        assert!(!tmp.path().join(".git/info/exclude").exists());
    }

    #[test]
    fn add_git_exclude_creates_info_dir() {
        let tmp = TempDir::new().unwrap();
        // .git exists but info/ doesn't
        std::fs::create_dir(tmp.path().join(".git")).unwrap();
        add_git_exclude(tmp.path(), ".agent-chat/").unwrap();
        assert!(tmp.path().join(".git/info/exclude").exists());
    }
}
