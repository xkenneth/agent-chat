use std::fs;
use std::path::Path;
use crate::error::Result;

const START_SENTINEL: &str = "<!-- agent-chat:start -->";
const END_SENTINEL: &str = "<!-- agent-chat:end -->";

const GUIDANCE: &str = r#"<!-- agent-chat:start -->
# Agent Chat

You are collaborating with other agents on this project. You were auto-registered
at session start — your name is in `$AGENT_CHAT_NAME`. Use it when referring to yourself.

## Commands

- `agent-chat say <msg>` — post to the shared chatroom
- `agent-chat read` — check for messages from other agents
- `agent-chat lock <glob>` — claim advisory file lock before editing
- `agent-chat unlock <glob>` — release when done
- `agent-chat locks` — see who's locked what

## Workflow

**Starting a task:**
1. Run `agent-chat read` to catch up on any messages
2. Say what you're about to work on: `agent-chat say "starting on auth middleware"`
3. Lock files you'll edit: `agent-chat lock "src/auth/**/*.rs"`

**While working:**
- Don't stop to wait for replies. If you've asked a question or are waiting on
  another agent, move to your next task. The Stop hook will notify you when a
  message arrives at the end of your turn.
- If the Stop hook shows unread messages, run `agent-chat read` on your next turn.

**Finishing a task:**
1. Unlock your files: `agent-chat unlock "src/auth/**/*.rs"`
2. Announce completion: `agent-chat say "auth middleware done, tests passing"`
3. Run `agent-chat read` to check if anything came in while you were working

**When blocked:**
- Say so: `agent-chat say "blocked on DB schema — need table layout from bold-hawk"`
- Move to a different task instead of waiting
- Check back with `agent-chat read` between tasks

## Message style

Keep messages short and actionable. Other agents pay tokens to read them.

- Good: `agent-chat say "lock conflict on src/api.rs — I'll take src/models.rs instead"`
- Bad: `agent-chat say "I noticed that the file src/api.rs appears to be locked by another agent, so I have decided to work on a different file instead, specifically src/models.rs"`

## File locking

Locks are advisory and expire after 5 minutes. Lock before multi-file edits,
unlock immediately when done. If `check-lock` warns you about a locked file,
coordinate with the lock owner before editing — don't just ignore the warning.
<!-- agent-chat:end -->"#;

/// Install or update the agent-chat section in CLAUDE.md.
/// - No CLAUDE.md: create it with just the agent-chat section
/// - CLAUDE.md exists with sentinel: replace that section
/// - CLAUDE.md exists without sentinel: append the section
pub fn install_claude_md(project_root: &Path) -> Result<()> {
    let path = project_root.join("CLAUDE.md");

    if !path.exists() {
        let tmp = project_root.join(".tmp.CLAUDE.md");
        fs::write(&tmp, GUIDANCE)?;
        fs::rename(&tmp, &path)?;
        return Ok(());
    }

    let existing = fs::read_to_string(&path)?;

    let new_content = if let Some(start) = existing.find(START_SENTINEL) {
        if let Some(end) = existing.find(END_SENTINEL) {
            // Replace existing section
            let before = &existing[..start];
            let after = &existing[end + END_SENTINEL.len()..];
            format!("{}{}{}", before.trim_end(), if before.is_empty() { "" } else { "\n\n" }, format!("{}{}", GUIDANCE, after))
        } else {
            // Malformed: has start but no end. Replace from start to EOF.
            let before = existing[..start].trim_end();
            if before.is_empty() {
                GUIDANCE.to_string()
            } else {
                format!("{}\n\n{}", before, GUIDANCE)
            }
        }
    } else {
        // No existing section: append
        let trimmed = existing.trim_end();
        if trimmed.is_empty() {
            GUIDANCE.to_string()
        } else {
            format!("{}\n\n{}\n", trimmed, GUIDANCE)
        }
    };

    let tmp = project_root.join(".tmp.CLAUDE.md");
    fs::write(&tmp, &new_content)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_new_claude_md() {
        let tmp = TempDir::new().unwrap();
        install_claude_md(tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
        assert!(content.contains(START_SENTINEL));
        assert!(content.contains(END_SENTINEL));
        assert!(content.contains("agent-chat read"));
    }

    #[test]
    fn appends_to_existing_claude_md() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(&path, "# My Project\n\nExisting content here.\n").unwrap();

        install_claude_md(tmp.path()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("# My Project"));
        assert!(content.contains("Existing content here."));
        assert!(content.contains(START_SENTINEL));
        assert!(content.contains("agent-chat read"));
    }

    #[test]
    fn replaces_existing_section() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        let old = format!(
            "# My Project\n\nStuff above.\n\n{}\n# Old Agent Chat\nold content\n{}\n\nStuff below.\n",
            START_SENTINEL, END_SENTINEL
        );
        fs::write(&path, &old).unwrap();

        install_claude_md(tmp.path()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Stuff above."));
        assert!(content.contains("Stuff below."));
        assert!(!content.contains("old content"));
        assert!(content.contains("agent-chat read"));
        // Only one sentinel pair
        assert_eq!(content.matches(START_SENTINEL).count(), 1);
        assert_eq!(content.matches(END_SENTINEL).count(), 1);
    }

    #[test]
    fn idempotent() {
        let tmp = TempDir::new().unwrap();
        install_claude_md(tmp.path()).unwrap();
        install_claude_md(tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
        assert_eq!(content.matches(START_SENTINEL).count(), 1);
        assert_eq!(content.matches(END_SENTINEL).count(), 1);
    }

    #[test]
    fn preserves_content_before_and_after() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(&path, "# Header\n\nBefore.\n").unwrap();

        install_claude_md(tmp.path()).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("# Header"));
        assert!(content.contains("Before."));

        // Run again — still preserved
        install_claude_md(tmp.path()).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("# Header"));
        assert!(content.contains("Before."));
        assert_eq!(content.matches(START_SENTINEL).count(), 1);
    }
}
