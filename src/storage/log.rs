use std::fs;
use std::path::Path;
use std::time::SystemTime;
use crate::error::Result;

/// Write a message to the log directory using tmp+rename for atomicity.
/// Filename: {timestamp_ns}.md
pub fn write_message(log_dir: &Path, name: &str, body: &str) -> Result<()> {
    let timestamp_ns = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let filename = format!("{}.md", timestamp_ns);
    let target = log_dir.join(&filename);
    let tmp = log_dir.join(format!(".tmp.{}", filename));

    let content = format!("name: {}\n{}\n", name, body);
    fs::write(&tmp, &content)?;
    fs::rename(&tmp, &target)?;
    Ok(())
}

/// List message files sorted by filename (chronological order).
/// Returns (filename, full_path) pairs.
pub fn list_messages(log_dir: &Path) -> Result<Vec<(String, std::path::PathBuf)>> {
    let mut entries = Vec::new();

    if !log_dir.exists() {
        return Ok(entries);
    }

    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".md") && !name.starts_with(".tmp.") {
            entries.push((name, entry.path()));
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}

/// Check if the log directory has any messages.
pub fn has_any_messages(log_dir: &Path) -> Result<bool> {
    if !log_dir.exists() {
        return Ok(false);
    }
    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".md") && !name.starts_with(".tmp.") {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn write_and_list_messages() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("log");
        fs::create_dir(&log).unwrap();

        write_message(&log, "swift-fox", "hello").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        write_message(&log, "bold-hawk", "world").unwrap();

        let msgs = list_messages(&log).unwrap();
        assert_eq!(msgs.len(), 2);
        // Should be in chronological order
        assert!(msgs[0].0 < msgs[1].0);
    }

    #[test]
    fn has_any_messages_empty() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("log");
        fs::create_dir(&log).unwrap();
        assert!(!has_any_messages(&log).unwrap());
    }

    #[test]
    fn has_any_messages_with_content() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("log");
        fs::create_dir(&log).unwrap();
        write_message(&log, "test", "msg").unwrap();
        assert!(has_any_messages(&log).unwrap());
    }
}
