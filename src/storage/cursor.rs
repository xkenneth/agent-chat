use std::fs;
use std::path::Path;
use filetime::{self, FileTime};
use crate::error::Result;

/// Get the cursor file path for a given session.
pub fn cursor_path(cursors_dir: &Path, session_id: &str) -> std::path::PathBuf {
    cursors_dir.join(session_id)
}

/// Check if there are unread messages by comparing mtimes.
/// Returns true if log_dir mtime > cursor mtime, or if cursor doesn't exist and log has entries.
pub fn has_unread(log_dir: &Path, cursor_file: &Path) -> Result<bool> {
    if !cursor_file.exists() {
        // No cursor: check if log dir has any entries
        return crate::storage::log::has_any_messages(log_dir);
    }

    let log_meta = fs::metadata(log_dir)?;
    let cursor_meta = fs::metadata(cursor_file)?;

    let log_mtime = FileTime::from_last_modification_time(&log_meta);
    let cursor_mtime = FileTime::from_last_modification_time(&cursor_meta);

    Ok(log_mtime > cursor_mtime)
}

/// Count unread messages (messages newer than cursor mtime).
pub fn count_unread(log_dir: &Path, cursor_file: &Path) -> Result<usize> {
    let messages = crate::storage::log::list_messages(log_dir)?;

    if !cursor_file.exists() {
        return Ok(messages.len());
    }

    let cursor_meta = fs::metadata(cursor_file)?;
    let cursor_mtime = FileTime::from_last_modification_time(&cursor_meta);

    let mut count = 0;
    for (_name, path) in &messages {
        if let Ok(meta) = fs::metadata(path) {
            let msg_mtime = FileTime::from_last_modification_time(&meta);
            if msg_mtime > cursor_mtime {
                count += 1;
            }
        }
    }
    Ok(count)
}

/// Advance the cursor to "now" by touching the cursor file.
pub fn advance(cursor_file: &Path) -> Result<()> {
    // Create or update the cursor file
    if !cursor_file.exists() {
        fs::write(cursor_file, "")?;
    }
    let now = FileTime::now();
    filetime::set_file_mtime(cursor_file, now)?;
    Ok(())
}

/// Get messages that are unread (newer than cursor mtime).
/// If no cursor exists, returns the last `default_count` messages.
pub fn get_unread_messages(
    log_dir: &Path,
    cursor_file: &Path,
    default_count: usize,
) -> Result<Vec<std::path::PathBuf>> {
    let messages = crate::storage::log::list_messages(log_dir)?;

    if !cursor_file.exists() {
        // First session: show last N messages
        let start = messages.len().saturating_sub(default_count);
        return Ok(messages[start..].iter().map(|(_, p)| p.clone()).collect());
    }

    let cursor_meta = fs::metadata(cursor_file)?;
    let cursor_mtime = FileTime::from_last_modification_time(&cursor_meta);

    let mut unread = Vec::new();
    for (_name, path) in &messages {
        if let Ok(meta) = fs::metadata(path) {
            let msg_mtime = FileTime::from_last_modification_time(&meta);
            if msg_mtime > cursor_mtime {
                unread.push(path.clone());
            }
        }
    }
    Ok(unread)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::storage::log::write_message;

    #[test]
    fn has_unread_no_cursor_no_messages() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("log");
        fs::create_dir(&log).unwrap();
        let cursor = tmp.path().join("cursor");

        assert!(!has_unread(&log, &cursor).unwrap());
    }

    #[test]
    fn has_unread_no_cursor_with_messages() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("log");
        fs::create_dir(&log).unwrap();
        let cursor = tmp.path().join("cursor");

        write_message(&log, "test", "hello").unwrap();
        assert!(has_unread(&log, &cursor).unwrap());
    }

    #[test]
    fn has_unread_after_advance() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("log");
        fs::create_dir(&log).unwrap();
        let cursor = tmp.path().join("cursor");

        write_message(&log, "test", "hello").unwrap();
        advance(&cursor).unwrap();

        // After advancing, should not have unread
        // (unless a new message was written in the same instant)
        std::thread::sleep(std::time::Duration::from_millis(50));
        // No new messages, so should be false
        // Note: on some filesystems mtime granularity may cause this to be flaky
        // but with the sleep it should be reliable
        assert!(!has_unread(&log, &cursor).unwrap());
    }

    #[test]
    fn get_unread_first_session_returns_last_n() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("log");
        fs::create_dir(&log).unwrap();
        let cursor = tmp.path().join("cursor");

        for i in 0..10 {
            write_message(&log, "test", &format!("msg {}", i)).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let unread = get_unread_messages(&log, &cursor, 5).unwrap();
        assert_eq!(unread.len(), 5);
    }
}
