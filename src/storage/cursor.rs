use std::fs;
use std::path::Path;
use filetime::{self, FileTime};
use crate::error::Result;
use crate::format;

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
/// If `exclude_name` is Some, skip messages authored by that name.
pub fn count_unread(log_dir: &Path, cursor_file: &Path, exclude_name: Option<&str>) -> Result<usize> {
    let messages = crate::storage::log::list_messages(log_dir)?;

    if !cursor_file.exists() {
        return Ok(count_excluding(&messages, exclude_name));
    }

    let cursor_meta = fs::metadata(cursor_file)?;
    let cursor_mtime = FileTime::from_last_modification_time(&cursor_meta);

    let mut count = 0;
    for (_name, path) in &messages {
        if let Ok(meta) = fs::metadata(path) {
            let msg_mtime = FileTime::from_last_modification_time(&meta);
            if msg_mtime > cursor_mtime {
                if should_include(path, exclude_name) {
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

/// Check if a message file should be included (not authored by exclude_name).
fn should_include(path: &Path, exclude_name: Option<&str>) -> bool {
    let exclude = match exclude_name {
        Some(name) => name,
        None => return true,
    };
    match fs::read_to_string(path) {
        Ok(content) => match format::parse_message_file(&content) {
            Some((name, _)) => name != exclude,
            None => true,
        },
        Err(_) => true,
    }
}

/// Count messages in a list, excluding those authored by exclude_name.
fn count_excluding(messages: &[(String, std::path::PathBuf)], exclude_name: Option<&str>) -> usize {
    messages.iter().filter(|(_, path)| should_include(path, exclude_name)).count()
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
/// If `exclude_name` is Some, skip messages authored by that name.
pub fn get_unread_messages(
    log_dir: &Path,
    cursor_file: &Path,
    default_count: usize,
    exclude_name: Option<&str>,
) -> Result<Vec<std::path::PathBuf>> {
    let messages = crate::storage::log::list_messages(log_dir)?;

    if !cursor_file.exists() {
        // First session: show last N messages, filtered
        let filtered: Vec<_> = messages
            .iter()
            .filter(|(_, path)| should_include(path, exclude_name))
            .map(|(_, p)| p.clone())
            .collect();
        let start = filtered.len().saturating_sub(default_count);
        return Ok(filtered[start..].to_vec());
    }

    let cursor_meta = fs::metadata(cursor_file)?;
    let cursor_mtime = FileTime::from_last_modification_time(&cursor_meta);

    let mut unread = Vec::new();
    for (_name, path) in &messages {
        if let Ok(meta) = fs::metadata(path) {
            let msg_mtime = FileTime::from_last_modification_time(&meta);
            if msg_mtime > cursor_mtime && should_include(path, exclude_name) {
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

        let unread = get_unread_messages(&log, &cursor, 5, None).unwrap();
        assert_eq!(unread.len(), 5);
    }

    #[test]
    fn count_unread_excludes_own_messages() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("log");
        fs::create_dir(&log).unwrap();
        let cursor = tmp.path().join("cursor");

        // Advance cursor first so all messages are "new"
        advance(&cursor).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        write_message(&log, "other-agent", "msg 1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        write_message(&log, "me", "msg 2").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        write_message(&log, "other-agent", "msg 3").unwrap();

        assert_eq!(count_unread(&log, &cursor, Some("me")).unwrap(), 2);
    }

    #[test]
    fn count_unread_no_filter_counts_all() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("log");
        fs::create_dir(&log).unwrap();
        let cursor = tmp.path().join("cursor");

        advance(&cursor).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        write_message(&log, "other-agent", "msg 1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        write_message(&log, "me", "msg 2").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        write_message(&log, "other-agent", "msg 3").unwrap();

        assert_eq!(count_unread(&log, &cursor, None).unwrap(), 3);
    }

    #[test]
    fn get_unread_excludes_own_messages() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("log");
        fs::create_dir(&log).unwrap();
        let cursor = tmp.path().join("cursor");

        advance(&cursor).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        write_message(&log, "other-agent", "msg 1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        write_message(&log, "me", "my msg").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        write_message(&log, "other-agent", "msg 3").unwrap();

        let unread = get_unread_messages(&log, &cursor, 5, Some("me")).unwrap();
        assert_eq!(unread.len(), 2);
        // Verify none of the returned paths contain "me" as author
        for path in &unread {
            let content = fs::read_to_string(path).unwrap();
            let (name, _) = format::parse_message_file(&content).unwrap();
            assert_ne!(name, "me");
        }
    }

    #[test]
    fn get_unread_first_session_excludes_own() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("log");
        fs::create_dir(&log).unwrap();
        let cursor = tmp.path().join("cursor");
        // No cursor — first session path

        for i in 0..5 {
            write_message(&log, "other-agent", &format!("msg {}", i)).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        write_message(&log, "me", "my msg").unwrap();

        let unread = get_unread_messages(&log, &cursor, 10, Some("me")).unwrap();
        assert_eq!(unread.len(), 5);
        for path in &unread {
            let content = fs::read_to_string(path).unwrap();
            let (name, _) = format::parse_message_file(&content).unwrap();
            assert_ne!(name, "me");
        }
    }
}
