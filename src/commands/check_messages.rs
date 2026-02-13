use std::path::Path;
use serde_json::json;
use crate::error::Result;
use crate::format;
use crate::storage::{cursor, identity, paths};

const DEFAULT_FIRST_READ_COUNT: usize = 5;

/// PreToolUse hook: inject unread messages into agent context via additionalContext.
/// Advances the cursor so the same messages aren't delivered again.
pub fn run(root: &Path) -> Result<()> {
    let id = match identity::resolve(root) {
        Ok(id) => id,
        Err(_) => return Ok(()),
    };

    // Filter out own messages so agents don't get nudged about their own posts
    let exclude = id.name.as_deref();

    let log_dir = paths::log_dir(root);
    let cursors_dir = paths::cursors_dir(root);
    let cursor_file = cursor::cursor_path(&cursors_dir, &id.session_id);

    let message_paths = cursor::get_unread_messages(&log_dir, &cursor_file, DEFAULT_FIRST_READ_COUNT, exclude)?;

    if message_paths.is_empty() {
        return Ok(());
    }

    let formatted = format::format_messages_from_paths(&message_paths);
    if formatted.is_empty() {
        return Ok(());
    }

    let output = json!({
        "hookSpecificOutput": {
            "additionalContext": formatted
        }
    });
    print!("{}", serde_json::to_string(&output)?);

    // Advance cursor so the same messages aren't delivered again
    cursor::advance(&cursor_file)?;

    Ok(())
}
