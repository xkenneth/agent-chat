use chrono::NaiveDateTime;

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

/// Format the status output for unread messages
pub fn format_status(unread_count: usize) -> String {
    if unread_count == 0 {
        String::new()
    } else if unread_count == 1 {
        "[agent-chat: 1 unread message]".to_string()
    } else {
        format!("[agent-chat: {} unread messages]", unread_count)
    }
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
    fn test_format_status() {
        assert_eq!(format_status(0), "");
        assert_eq!(format_status(1), "[agent-chat: 1 unread message]");
        assert_eq!(format_status(5), "[agent-chat: 5 unread messages]");
    }
}
