use crate::error::{AgentChatError, Result};
use crate::storage::{paths, session};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Identity {
    pub session_id: String,
    pub name: Option<String>,
}

/// Resolve identity with env-first semantics and Codex-friendly fallbacks.
/// 1) Use AGENT_CHAT_SESSION_ID / AGENT_CHAT_NAME when present.
/// 2) If session_id exists but name is missing, read name from sessions/<session_id>.
/// 3) If session_id is missing and exactly one session file exists, use that session id.
pub fn resolve(root: &Path) -> Result<Identity> {
    let env_session = std::env::var("AGENT_CHAT_SESSION_ID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let env_name = std::env::var("AGENT_CHAT_NAME")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let session_id = match env_session {
        Some(sid) => sid,
        None => infer_single_session_id(root)?.ok_or_else(|| {
            AgentChatError::MissingEnv("AGENT_CHAT_SESSION_ID".to_string())
        })?,
    };

    let name = match env_name {
        Some(name) => Some(name),
        None => {
            let sessions_dir = paths::sessions_dir(root);
            session::read_session(&sessions_dir, &session_id)?
        }
    };

    Ok(Identity { session_id, name })
}

pub fn require_name(identity: &Identity) -> Result<&str> {
    identity
        .name
        .as_deref()
        .ok_or_else(|| AgentChatError::MissingEnv("AGENT_CHAT_NAME".to_string()))
}

fn infer_single_session_id(root: &Path) -> Result<Option<String>> {
    let sessions_dir = paths::sessions_dir(root);
    if !sessions_dir.exists() {
        return Ok(None);
    }

    let mut ids = Vec::new();
    for entry in std::fs::read_dir(&sessions_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.starts_with(".tmp.") {
            continue;
        }
        ids.push(file_name);
        if ids.len() > 1 {
            return Ok(None);
        }
    }

    Ok(ids.into_iter().next())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::session;
    use tempfile::TempDir;

    #[test]
    fn infer_single_session_returns_none_when_missing() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".agent-chat");
        std::fs::create_dir_all(&root).unwrap();
        assert_eq!(infer_single_session_id(&root).unwrap(), None);
    }

    #[test]
    fn infer_single_session_returns_id() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".agent-chat");
        let sessions = root.join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        session::write_session(&sessions, "sid-1", "swift-fox").unwrap();
        assert_eq!(
            infer_single_session_id(&root).unwrap(),
            Some("sid-1".to_string())
        );
    }

    #[test]
    fn infer_single_session_returns_none_when_multiple() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".agent-chat");
        let sessions = root.join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        session::write_session(&sessions, "sid-1", "swift-fox").unwrap();
        session::write_session(&sessions, "sid-2", "bold-hawk").unwrap();
        assert_eq!(infer_single_session_id(&root).unwrap(), None);
    }
}
