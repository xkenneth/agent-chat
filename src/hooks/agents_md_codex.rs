use std::fs;
use std::path::Path;
use crate::error::Result;

const START_SENTINEL: &str = "<!-- agent-chat-codex:start -->";
const END_SENTINEL: &str = "<!-- agent-chat-codex:end -->";

const GUIDANCE: &str = r#"<!-- agent-chat-codex:start -->
## Agent Chat (Codex)

Use `agent-chat` for inter-agent coordination in this repo.

### Commands

- `agent-chat register --session-id <id>` — initialize identity for this Codex session
- `agent-chat read` — check unread messages from other agents
- `agent-chat say "<msg>"` — post short status updates
- `agent-chat lock "<glob>"` — advisory lock before editing shared files
- `agent-chat unlock "<glob>"` — release lock immediately after edits
- `agent-chat locks` — inspect active locks
- `agent-chat focus "<area>"` — declare active focus area
- `agent-chat focus --clear` — clear focus when done
- `agent-chat focuses` — inspect active focuses

### Suggested startup

1. Register once per Codex session: `agent-chat register --session-id "$USER-$(date +%s)"`
2. Run `agent-chat read`
3. Announce scope: `agent-chat say "starting on <task>"`
4. Lock planned files: `agent-chat lock "src/<area>/**"`
5. Set focus: `agent-chat focus "<area>"`

### While working

- Run `agent-chat read` every few tool calls.
- Keep messages short and actionable.
- If you are blocked, say it and move to another task.

### Finishing

1. Unlock files you touched.
2. Clear focus.
3. Announce completion.
4. Run `agent-chat read` once more.
<!-- agent-chat-codex:end -->"#;

/// Install or update the agent-chat Codex section in `<target_dir>/AGENTS.md`.
pub fn install_agents_md_to(target_dir: &Path) -> Result<()> {
    fs::create_dir_all(target_dir)?;
    let path = target_dir.join("AGENTS.md");

    if !path.exists() {
        let tmp = target_dir.join(".tmp.AGENTS.md");
        fs::write(&tmp, GUIDANCE)?;
        fs::rename(&tmp, &path)?;
        return Ok(());
    }

    let existing = fs::read_to_string(&path)?;

    let new_content = if let Some(start) = existing.find(START_SENTINEL) {
        if let Some(end) = existing.find(END_SENTINEL) {
            let before = &existing[..start];
            let after = &existing[end + END_SENTINEL.len()..];
            format!(
                "{}{}{}{}",
                before.trim_end(),
                if before.is_empty() { "" } else { "\n\n" },
                GUIDANCE,
                after
            )
        } else {
            let before = existing[..start].trim_end();
            if before.is_empty() {
                GUIDANCE.to_string()
            } else {
                format!("{}\n\n{}", before, GUIDANCE)
            }
        }
    } else {
        let trimmed = existing.trim_end();
        if trimmed.is_empty() {
            GUIDANCE.to_string()
        } else {
            format!("{}\n\n{}\n", trimmed, GUIDANCE)
        }
    };

    let tmp = target_dir.join(".tmp.AGENTS.md");
    fs::write(&tmp, &new_content)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_new_agents_md() {
        let tmp = TempDir::new().unwrap();
        install_agents_md_to(tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
        assert!(content.contains(START_SENTINEL));
        assert!(content.contains(END_SENTINEL));
        assert!(content.contains("agent-chat register --session-id"));
    }

    #[test]
    fn appends_to_existing_agents_md() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("AGENTS.md");
        fs::write(&path, "# Project Agents\n\nExisting guidance.\n").unwrap();

        install_agents_md_to(tmp.path()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("# Project Agents"));
        assert!(content.contains("Existing guidance."));
        assert!(content.contains(START_SENTINEL));
    }

    #[test]
    fn idempotent() {
        let tmp = TempDir::new().unwrap();
        install_agents_md_to(tmp.path()).unwrap();
        install_agents_md_to(tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
        assert_eq!(content.matches(START_SENTINEL).count(), 1);
        assert_eq!(content.matches(END_SENTINEL).count(), 1);
    }
}
