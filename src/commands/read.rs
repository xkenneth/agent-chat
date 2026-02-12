use std::fs;
use std::path::Path;
use chrono::{DateTime, Local, NaiveDateTime};
use crate::error::{AgentChatError, Result};
use crate::format;
use crate::storage::{cursor, log, paths};

const DEFAULT_FIRST_READ_COUNT: usize = 5;

pub fn run(root: &Path, show_all: bool) -> Result<()> {
    let session_id = std::env::var("AGENT_CHAT_SESSION_ID")
        .map_err(|_| AgentChatError::MissingEnv("AGENT_CHAT_SESSION_ID".to_string()))?;

    let log_dir = paths::log_dir(root);
    let cursors_dir = paths::cursors_dir(root);
    let cursor_file = cursor::cursor_path(&cursors_dir, &session_id);

    let message_paths = if show_all {
        let msgs = log::list_messages(&log_dir)?;
        msgs.into_iter().map(|(_, p)| p).collect()
    } else {
        cursor::get_unread_messages(&log_dir, &cursor_file, DEFAULT_FIRST_READ_COUNT)?
    };

    for path in &message_paths {
        if let Ok(content) = fs::read_to_string(path) {
            if let Some((name, body)) = format::parse_message_file(&content) {
                // Extract timestamp from filename
                let filename = path.file_stem().unwrap().to_string_lossy();
                let ts = parse_timestamp_ns(&filename);
                println!("{}", format::format_message(name, ts, body));
            }
        }
    }

    // Advance cursor after reading
    if !message_paths.is_empty() || !cursor_file.exists() {
        cursor::advance(&cursor_file)?;
    }

    Ok(())
}

/// Parse nanosecond timestamp from filename to NaiveDateTime
fn parse_timestamp_ns(filename: &str) -> NaiveDateTime {
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
