use std::fs;
use std::path::Path;
use crate::error::Result;

/// Write a session mapping: session_id -> friendly_name
pub fn write_session(sessions_dir: &Path, session_id: &str, name: &str) -> Result<()> {
    let path = sessions_dir.join(session_id);
    let tmp = sessions_dir.join(format!(".tmp.{}", session_id));
    fs::write(&tmp, name)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Read the friendly name for a session_id. Returns None if not registered.
pub fn read_session(sessions_dir: &Path, session_id: &str) -> Result<Option<String>> {
    let path = sessions_dir.join(session_id);
    if !path.exists() {
        return Ok(None);
    }
    let name = fs::read_to_string(&path)?.trim().to_string();
    Ok(Some(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn write_and_read_session() {
        let tmp = TempDir::new().unwrap();
        write_session(tmp.path(), "abc123", "swift-fox").unwrap();
        let name = read_session(tmp.path(), "abc123").unwrap();
        assert_eq!(name, Some("swift-fox".to_string()));
    }

    #[test]
    fn read_missing_session() {
        let tmp = TempDir::new().unwrap();
        let name = read_session(tmp.path(), "missing").unwrap();
        assert_eq!(name, None);
    }
}
