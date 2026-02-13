use std::path::Path;
use serde_json::json;
use crate::error::Result;
use crate::format;
use crate::storage::{cursor, identity, paths};

const DEFAULT_FIRST_READ_COUNT: usize = 10;

pub fn run(root: &Path) -> Result<()> {
    let log_dir = paths::log_dir(root);

    let id = match identity::resolve(root) {
        Ok(id) => id,
        Err(_) => return Ok(()),
    };
    let session_id = id.session_id.as_str();
    let exclude = id.name.as_deref();

    let cursors_dir = paths::cursors_dir(root);
    let cursor_file = cursor::cursor_path(&cursors_dir, session_id);
    let has_unread = cursor::has_unread(&log_dir, &cursor_file)?;

    if !has_unread {
        return Ok(());
    }

    // Get unread message paths
    let message_paths = cursor::get_unread_messages(
        &log_dir,
        &cursor_file,
        DEFAULT_FIRST_READ_COUNT,
        exclude,
    )?;

    if message_paths.is_empty() {
        return Ok(());
    }

    let formatted = format::format_messages_for_status(&message_paths);
    if formatted.is_empty() {
        return Ok(());
    }

    // Output decision:block JSON — prevents agent from stopping without reading
    // Do NOT advance cursor — agent should run `agent-chat read` to formally process
    let output = json!({
        "decision": "block",
        "reason": formatted
    });
    print!("{}", serde_json::to_string(&output)?);

    Ok(())
}
