use std::path::Path;
use serde_json::json;
use crate::error::Result;
use crate::format;
use crate::storage::{cursor, paths};

const DEFAULT_FIRST_READ_COUNT: usize = 10;

pub fn run(root: &Path) -> Result<()> {
    let log_dir = paths::log_dir(root);

    // Try to get session_id from env; if missing, just check if any messages exist
    let session_id = std::env::var("AGENT_CHAT_SESSION_ID").ok();
    // Filter out own messages so agents don't get nudged about their own posts
    let my_name = std::env::var("AGENT_CHAT_NAME").ok();
    let exclude = my_name.as_deref();

    let has_unread = if let Some(ref sid) = session_id {
        let cursors_dir = paths::cursors_dir(root);
        let cursor_file = cursor::cursor_path(&cursors_dir, sid);
        cursor::has_unread(&log_dir, &cursor_file)?
    } else {
        // No session: check if any messages exist at all
        crate::storage::log::has_any_messages(&log_dir)?
    };

    if !has_unread {
        return Ok(());
    }

    // Get unread message paths
    let message_paths = if let Some(ref sid) = session_id {
        let cursors_dir = paths::cursors_dir(root);
        let cursor_file = cursor::cursor_path(&cursors_dir, sid);
        cursor::get_unread_messages(&log_dir, &cursor_file, DEFAULT_FIRST_READ_COUNT, exclude)?
    } else {
        let msgs = crate::storage::log::list_messages(&log_dir)?;
        msgs.into_iter().map(|(_, p)| p).collect()
    };

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
