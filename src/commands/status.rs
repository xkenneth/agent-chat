use std::path::Path;
use crate::error::Result;
use crate::format;
use crate::storage::{cursor, paths};

pub fn run(root: &Path) -> Result<()> {
    let log_dir = paths::log_dir(root);

    // Try to get session_id from env; if missing, just check if any messages exist
    let session_id = std::env::var("AGENT_CHAT_SESSION_ID").ok();

    let has_unread = if let Some(ref sid) = session_id {
        let cursors_dir = paths::cursors_dir(root);
        let cursor_file = cursor::cursor_path(&cursors_dir, sid);
        cursor::has_unread(&log_dir, &cursor_file)?
    } else {
        // No session: check if any messages exist at all
        crate::storage::log::has_any_messages(&log_dir)?
    };

    if has_unread {
        let count = if let Some(ref sid) = session_id {
            let cursors_dir = paths::cursors_dir(root);
            let cursor_file = cursor::cursor_path(&cursors_dir, sid);
            cursor::count_unread(&log_dir, &cursor_file)?
        } else {
            let msgs = crate::storage::log::list_messages(&log_dir)?;
            msgs.len()
        };
        let status = format::format_status(count);
        if !status.is_empty() {
            print!("{}", status);
        }
    }
    // When no unread: print nothing (zero tokens)

    Ok(())
}
