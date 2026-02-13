use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct FocusEntry {
    pub focus: String,
    pub owner: String,
    pub session_id: String,
    pub set_at: u64, // unix epoch seconds
    pub ttl_secs: u64,
}

impl FocusEntry {
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.set_at + self.ttl_secs
    }
}

fn focus_path(focuses_dir: &Path, session_id: &str) -> PathBuf {
    focuses_dir.join(format!("{}.focus", session_id))
}

/// Set a focus for the given session. Replaces any previous focus.
pub fn set(
    focuses_dir: &Path,
    focus: &str,
    owner: &str,
    session_id: &str,
    ttl_secs: u64,
) -> Result<()> {
    cleanup_expired(focuses_dir)?;

    let entry = FocusEntry {
        focus: focus.to_string(),
        owner: owner.to_string(),
        session_id: session_id.to_string(),
        set_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        ttl_secs,
    };

    let content = serde_json::to_string_pretty(&entry)?;
    let path = focus_path(focuses_dir, session_id);
    let tmp = focuses_dir.join(format!(".tmp.{}.focus", session_id));
    fs::write(&tmp, &content)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Clear the focus for the given session.
pub fn clear(focuses_dir: &Path, session_id: &str) -> Result<()> {
    let path = focus_path(focuses_dir, session_id);
    let _ = fs::remove_file(&path); // ignore ENOENT
    Ok(())
}

/// List all active (non-expired) focuses.
pub fn list_active(focuses_dir: &Path) -> Result<Vec<FocusEntry>> {
    let mut focuses = Vec::new();
    if !focuses_dir.exists() {
        return Ok(focuses);
    }

    for entry in fs::read_dir(focuses_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".focus") || name.starts_with(".tmp.") {
            continue;
        }
        match fs::read_to_string(entry.path()) {
            Ok(content) => {
                if let Ok(focus) = serde_json::from_str::<FocusEntry>(&content) {
                    if !focus.is_expired() {
                        focuses.push(focus);
                    } else {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
            Err(_) => continue,
        }
    }
    Ok(focuses)
}

/// Stop words to skip when tokenizing for overlap detection.
const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for",
    "of", "with", "by", "from", "is", "it", "as", "be", "was", "are",
    "this", "that", "into", "all", "no", "not", "so", "up", "out",
];

/// Tokenize a string into lowercase significant words.
fn tokenize(text: &str) -> HashSet<String> {
    let stop: HashSet<&str> = STOP_WORDS.iter().copied().collect();
    text.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() > 1 && !stop.contains(w.as_str()))
        .collect()
}

/// Find focuses from other sessions that overlap with the given text.
pub fn find_overlapping(
    focuses_dir: &Path,
    text: &str,
    session_id: &str,
) -> Result<Vec<FocusEntry>> {
    let text_tokens = tokenize(text);
    if text_tokens.is_empty() {
        return Ok(Vec::new());
    }

    let focuses = list_active(focuses_dir)?;
    let mut overlapping = Vec::new();

    for focus in focuses {
        if focus.session_id == session_id {
            continue;
        }
        let focus_tokens = tokenize(&focus.focus);
        if !text_tokens.is_disjoint(&focus_tokens) {
            overlapping.push(focus);
        }
    }

    Ok(overlapping)
}

/// Clean up expired focus files.
fn cleanup_expired(focuses_dir: &Path) -> Result<()> {
    if !focuses_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(focuses_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".focus") || name.starts_with(".tmp.") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Ok(focus) = serde_json::from_str::<FocusEntry>(&content) {
                if focus.is_expired() {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn set_and_list() {
        let tmp = TempDir::new().unwrap();
        set(tmp.path(), "CI pipeline", "swift-fox", "sess1", 300).unwrap();
        let focuses = list_active(tmp.path()).unwrap();
        assert_eq!(focuses.len(), 1);
        assert_eq!(focuses[0].focus, "CI pipeline");
        assert_eq!(focuses[0].owner, "swift-fox");
    }

    #[test]
    fn set_replaces_previous() {
        let tmp = TempDir::new().unwrap();
        set(tmp.path(), "CI pipeline", "swift-fox", "sess1", 300).unwrap();
        set(tmp.path(), "API work", "swift-fox", "sess1", 300).unwrap();
        let focuses = list_active(tmp.path()).unwrap();
        assert_eq!(focuses.len(), 1);
        assert_eq!(focuses[0].focus, "API work");
    }

    #[test]
    fn clear_removes_focus() {
        let tmp = TempDir::new().unwrap();
        set(tmp.path(), "CI pipeline", "swift-fox", "sess1", 300).unwrap();
        clear(tmp.path(), "sess1").unwrap();
        let focuses = list_active(tmp.path()).unwrap();
        assert_eq!(focuses.len(), 0);
    }

    #[test]
    fn clear_nonexistent_ok() {
        let tmp = TempDir::new().unwrap();
        clear(tmp.path(), "sess1").unwrap(); // should not error
    }

    #[test]
    fn multiple_sessions() {
        let tmp = TempDir::new().unwrap();
        set(tmp.path(), "CI pipeline", "swift-fox", "sess1", 300).unwrap();
        set(tmp.path(), "API work", "bold-hawk", "sess2", 300).unwrap();
        let focuses = list_active(tmp.path()).unwrap();
        assert_eq!(focuses.len(), 2);
    }

    #[test]
    fn expired_focus_cleaned_up() {
        let tmp = TempDir::new().unwrap();
        set(tmp.path(), "CI pipeline", "swift-fox", "sess1", 0).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let focuses = list_active(tmp.path()).unwrap();
        assert_eq!(focuses.len(), 0);
    }

    #[test]
    fn find_overlapping_matches() {
        let tmp = TempDir::new().unwrap();
        set(tmp.path(), "CI pipeline", "swift-fox", "sess1", 300).unwrap();
        let overlaps = find_overlapping(tmp.path(), "CI configuration", "sess2").unwrap();
        assert_eq!(overlaps.len(), 1);
        assert_eq!(overlaps[0].owner, "swift-fox");
    }

    #[test]
    fn find_overlapping_skips_own_session() {
        let tmp = TempDir::new().unwrap();
        set(tmp.path(), "CI pipeline", "swift-fox", "sess1", 300).unwrap();
        let overlaps = find_overlapping(tmp.path(), "CI configuration", "sess1").unwrap();
        assert_eq!(overlaps.len(), 0);
    }

    #[test]
    fn find_overlapping_no_match() {
        let tmp = TempDir::new().unwrap();
        set(tmp.path(), "CI pipeline", "swift-fox", "sess1", 300).unwrap();
        let overlaps = find_overlapping(tmp.path(), "database migration", "sess2").unwrap();
        assert_eq!(overlaps.len(), 0);
    }

    #[test]
    fn tokenize_filters_stop_words() {
        let tokens = tokenize("work on the CI pipeline for testing");
        assert!(tokens.contains("ci"));
        assert!(tokens.contains("pipeline"));
        assert!(tokens.contains("work"));
        assert!(tokens.contains("testing"));
        assert!(!tokens.contains("the"));
        assert!(!tokens.contains("on"));
        assert!(!tokens.contains("for"));
    }
}
