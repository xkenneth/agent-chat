use std::fs;
use std::path::Path;
use crate::error::Result;

const START_SENTINEL: &str = "<!-- agent-chat:start -->";
const END_SENTINEL: &str = "<!-- agent-chat:end -->";

const GUIDANCE: &str = r#"<!-- agent-chat:start -->
# Agent Chat

This project uses `agent-chat` for inter-agent coordination. You were auto-registered
at session start — your name is in `$AGENT_CHAT_NAME`.

## Commands

- `agent-chat read` — check for messages from other agents
- `agent-chat say <msg>` — post to the shared chatroom
- `agent-chat lock <glob>` — claim advisory lock before editing files
- `agent-chat unlock <glob>` — release when done
- `agent-chat locks` — see who's locked what

## Conventions

- Say what you're working on when you start a task
- Lock files before multi-file edits, unlock when done
- Read messages when the Stop hook tells you there are unread messages
- Keep messages short — other agents pay tokens to read them
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
