use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Local, NaiveDateTime};

/// Format a message for display: [name HH:MM]: message
pub fn format_message(name: &str, timestamp: NaiveDateTime, body: &str) -> String {
    let time = timestamp.format("%H:%M");
    format!("[{} {}]: {}", name, time, body)
}

/// Parse a message file's content. Expected format:
/// First line: `name: <friendly_name>`
/// Remaining lines: message body
pub fn parse_message_file(content: &str) -> Option<(&str, &str)> {
    let first_newline = content.find('\n')?;
    let header = &content[..first_newline];
    let name = header.strip_prefix("name: ")?;
    let body = content[first_newline + 1..].trim_end();
    Some((name, body))
}

/// Parse nanosecond timestamp from filename to NaiveDateTime (local time).
pub fn parse_timestamp_ns(filename: &str) -> NaiveDateTime {
    if let Ok(ns) = filename.parse::<u128>() {
        let secs = (ns / 1_000_000_000) as i64;
        let nsecs = (ns % 1_000_000_000) as u32;
        DateTime::from_timestamp(secs, nsecs)
            .map(|dt| dt.with_timezone(&Local).naive_local())
            .unwrap_or_else(|| Local::now().naive_local())
    } else {
        Local::now().naive_local()
    }
}

/// Read message files from paths and format them as a message list with a header.
/// Returns empty string if no messages could be parsed.
pub fn format_messages_from_paths(paths: &[PathBuf]) -> String {
    let mut lines = Vec::new();
    for path in paths {
        if let Ok(content) = fs::read_to_string(path) {
            if let Some((name, body)) = parse_message_file(&content) {
                let filename = path.file_stem().unwrap().to_string_lossy();
                let ts = parse_timestamp_ns(&filename);
                lines.push(format_message(name, ts, body));
            }
        }
    }
    if lines.is_empty() {
        return String::new();
    }
    let count = lines.len();
    let header = if count == 1 {
        "[agent-chat: 1 unread message]".to_string()
    } else {
        format!("[agent-chat: {} unread messages]", count)
    };
    format!("{}\n{}", header, lines.join("\n"))
}

/// Format a path for use in status check — does NOT include cursor-advancing instructions.
pub fn format_messages_for_status(paths: &[PathBuf]) -> String {
    let formatted = format_messages_from_paths(paths);
    if formatted.is_empty() {
        return String::new();
    }
    format!("{}\nRun `agent-chat read` to acknowledge, then respond or continue.", formatted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_message() {
        let ts = NaiveDateTime::parse_from_str("2025-01-15 14:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let result = format_message("swift-fox", ts, "hello world");
        assert_eq!(result, "[swift-fox 14:30]: hello world");
    }

    #[test]
    fn test_parse_message_file() {
        let content = "name: swift-fox\nhello world";
        let (name, body) = parse_message_file(content).unwrap();
        assert_eq!(name, "swift-fox");
        assert_eq!(body, "hello world");
    }

    #[test]
    fn test_parse_message_file_multiline_body() {
        let content = "name: bold-hawk\nline one\nline two";
        let (name, body) = parse_message_file(content).unwrap();
        assert_eq!(name, "bold-hawk");
        assert_eq!(body, "line one\nline two");
    }

    #[test]
    fn test_format_messages_from_paths_empty() {
        let result = format_messages_from_paths(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_messages_from_paths_single() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("1736950200000000000.msg");
        std::fs::write(&path, "name: swift-fox\nhello world").unwrap();

        let result = format_messages_from_paths(&[path]);
        assert!(result.contains("[agent-chat: 1 unread message]"));
        assert!(result.contains("swift-fox"));
        assert!(result.contains("hello world"));
    }

    #[test]
    fn test_format_messages_from_paths_multiple() {
        let dir = tempfile::tempdir().unwrap();
        let p1 = dir.path().join("1736950200000000000.msg");
        let p2 = dir.path().join("1736950260000000000.msg");
        std::fs::write(&p1, "name: swift-fox\nmsg one").unwrap();
        std::fs::write(&p2, "name: bold-hawk\nmsg two").unwrap();

        let result = format_messages_from_paths(&[p1, p2]);
        assert!(result.contains("[agent-chat: 2 unread messages]"));
        assert!(result.contains("msg one"));
        assert!(result.contains("msg two"));
    }

    #[test]
    fn test_format_messages_for_status() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("1736950200000000000.msg");
        std::fs::write(&path, "name: swift-fox\nhello").unwrap();

        let result = format_messages_for_status(&[path]);
        assert!(result.contains("[agent-chat: 1 unread message]"));
        assert!(result.contains("hello"));
        assert!(result.contains("agent-chat read"));
    }

    #[test]
    fn test_format_messages_for_status_empty() {
        let result = format_messages_for_status(&[]);
        assert_eq!(result, "");
    }
}
