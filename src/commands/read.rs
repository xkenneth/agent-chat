use std::fs;
use std::path::Path;
use crate::error::{AgentChatError, Result};
use crate::format;
use crate::storage::{cursor, log, paths};

const DEFAULT_FIRST_READ_COUNT: usize = 5;

pub fn run(root: &Path, show_all: bool) -> Result<()> {
    let session_id = std::env::var("AGENT_CHAT_SESSION_ID")
        .map_err(|_| AgentChatError::MissingEnv("AGENT_CHAT_SESSION_ID".to_string()))?;

    // Filter out own messages to avoid wasting tokens
    let my_name = std::env::var("AGENT_CHAT_NAME").ok();
    let exclude = my_name.as_deref();

    let log_dir = paths::log_dir(root);
    let cursors_dir = paths::cursors_dir(root);
    let cursor_file = cursor::cursor_path(&cursors_dir, &session_id);

    let message_paths = if show_all {
        let msgs = log::list_messages(&log_dir)?;
        // Filter own messages for --all mode too
        msgs.into_iter()
            .filter(|(_, path)| {
                match exclude {
                    Some(name) => {
                        match fs::read_to_string(path) {
                            Ok(content) => match format::parse_message_file(&content) {
                                Some((author, _)) => author != name,
                                None => true,
                            },
                            Err(_) => true,
                        }
                    }
                    None => true,
                }
            })
            .map(|(_, p)| p)
            .collect()
    } else {
        cursor::get_unread_messages(&log_dir, &cursor_file, DEFAULT_FIRST_READ_COUNT, exclude)?
    };

    for path in &message_paths {
        if let Ok(content) = fs::read_to_string(path) {
            if let Some((name, body)) = format::parse_message_file(&content) {
                // Extract timestamp from filename
                let filename = path.file_stem().unwrap().to_string_lossy();
                let ts = format::parse_timestamp_ns(&filename);
                println!("{}", format::format_message(name, ts, body));
            }
        }
    }

    // Advance cursor after reading (always, even if all were own messages)
    // We advance based on ALL messages (including own) so the cursor moves past them
    cursor::advance(&cursor_file)?;

    Ok(())
}

