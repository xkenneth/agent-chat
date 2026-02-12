use std::fs;
use std::path::Path;
use crate::error::Result;

const BR_START_SENTINEL: &str = "<!-- agent-chat-br:start -->";
const BR_END_SENTINEL: &str = "<!-- agent-chat-br:end -->";

const BR_GUIDANCE: &str = r#"<!-- agent-chat-br:start -->
# Beads Issue Tracker (br)

This project uses `br` (beads_rust) for issue tracking. Issues live in `.beads/`.

## Rules — read these first

1. **Beads form the plan.** Before diving into code, break the goal into beads that
   form a coherent plan. Each bead should represent a meaningful deliverable, not
   every small task.
2. **Claim before starting:** `agent-chat br-claim <id>` before working on a bead.
3. **Complete when done:** `agent-chat br-complete <id> --reason "..."` as soon as
   a bead's work is finished. Don't leave beads open — close them so others can see progress.
4. **Beads are your memory.** If your context gets compacted or you restart, beads
   tell you what the plan is and where things stand. Write them so a fresh agent can
   pick up where you left off.

## Plan mode — design your beads

When working in plan mode:
1. Identify which beads need to be created as part of the plan
2. Write each bead so it can survive context compaction or a complete agent restart
3. Each bead MUST include:
   - **Why** — the motivation or problem being solved
   - **What success looks like** — concrete deliverables and acceptance criteria
   - **Key context** — file paths, function names, architectural decisions

After plan approval, create beads as the FIRST execution step:

    br create "Title" --description "Why: ... What: ... Files: ..." --priority 2

Set dependencies between beads when order matters:

    br dep add <child-id> <parent-id>

## Execution workflow

1. Find ready work: `br ready`
2. Claim: `agent-chat br-claim <id>` (sets in_progress + assignee + announces)
3. Do the work
4. Complete: `agent-chat br-complete <id> --reason "done, tests passing"`
5. Sync: `br sync --flush-only`
6. Commit: `git add .beads/ && git commit -m "beads: update issue state"`

## Common commands

| Command | Purpose |
|---------|---------|
| `br create "Title" --description "..."` | New issue |
| `br ready` | Actionable (unblocked, open) issues |
| `br list --status open` | All open issues |
| `br show <id>` | Full issue details |
| `br update <id> --priority 0` | Change priority (0=highest) |
| `br dep add <child> <parent>` | Add dependency |
| `br dep tree <id>` | Visualize dependency chain |
| `br sync --flush-only` | Export DB → JSONL (never auto-commits) |

**Note:** Ensure `Bash(br *)` is in your Claude Code permissions to allow direct br commands.
<!-- agent-chat-br:end -->"#;

/// Install or update the br section in `<target_dir>/CLAUDE.md`.
/// - No CLAUDE.md: create it with just the br section
/// - CLAUDE.md exists with sentinel: replace that section
/// - CLAUDE.md exists without sentinel: append the section
pub fn install_br_claude_md_to(target_dir: &Path) -> Result<()> {
    fs::create_dir_all(target_dir)?;
    let path = target_dir.join("CLAUDE.md");

    if !path.exists() {
        let tmp = target_dir.join(".tmp.CLAUDE.md");
        fs::write(&tmp, BR_GUIDANCE)?;
        fs::rename(&tmp, &path)?;
        return Ok(());
    }

    let existing = fs::read_to_string(&path)?;

    let new_content = if let Some(start) = existing.find(BR_START_SENTINEL) {
        if let Some(end) = existing.find(BR_END_SENTINEL) {
            // Replace existing section
            let before = &existing[..start];
            let after = &existing[end + BR_END_SENTINEL.len()..];
            format!("{}{}{}", before.trim_end(), if before.is_empty() { "" } else { "\n\n" }, format!("{}{}", BR_GUIDANCE, after))
        } else {
            // Malformed: has start but no end. Replace from start to EOF.
            let before = existing[..start].trim_end();
            if before.is_empty() {
                BR_GUIDANCE.to_string()
            } else {
                format!("{}\n\n{}", before, BR_GUIDANCE)
            }
        }
    } else {
        // No existing section: append
        let trimmed = existing.trim_end();
        if trimmed.is_empty() {
            BR_GUIDANCE.to_string()
        } else {
            format!("{}\n\n{}\n", trimmed, BR_GUIDANCE)
        }
    };

    let tmp = target_dir.join(".tmp.CLAUDE.md");
    fs::write(&tmp, &new_content)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Remove the br section from `<target_dir>/CLAUDE.md`.
/// No-ops if the file is missing or has no br section.
pub fn remove_br_claude_md_from(target_dir: &Path) -> Result<()> {
    let path = target_dir.join("CLAUDE.md");

    if !path.exists() {
        return Ok(());
    }

    let existing = fs::read_to_string(&path)?;

    let Some(start) = existing.find(BR_START_SENTINEL) else {
        return Ok(());
    };
    let Some(end) = existing.find(BR_END_SENTINEL) else {
        return Ok(());
    };

    let before = existing[..start].trim_end();
    let after = existing[end + BR_END_SENTINEL.len()..].trim_start();

    let new_content = match (before.is_empty(), after.is_empty()) {
        (true, true) => String::new(),
        (true, false) => after.to_string(),
        (false, true) => format!("{}\n", before),
        (false, false) => format!("{}\n\n{}\n", before, after),
    };

    if new_content.is_empty() {
        fs::remove_file(&path)?;
    } else {
        let tmp = target_dir.join(".tmp.CLAUDE.md");
        fs::write(&tmp, &new_content)?;
        fs::rename(&tmp, &path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_new_claude_md_with_br_section() {
        let tmp = TempDir::new().unwrap();
        install_br_claude_md_to(tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
        assert!(content.contains(BR_START_SENTINEL));
        assert!(content.contains(BR_END_SENTINEL));
        assert!(content.contains("br ready"));
    }

    #[test]
    fn appends_to_existing_claude_md() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(&path, "# My Project\n\nExisting content here.\n").unwrap();

        install_br_claude_md_to(tmp.path()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("# My Project"));
        assert!(content.contains("Existing content here."));
        assert!(content.contains(BR_START_SENTINEL));
        assert!(content.contains("br ready"));
    }

    #[test]
    fn replaces_existing_br_section() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        let old = format!(
            "# My Project\n\nStuff above.\n\n{}\n# Old BR\nold content\n{}\n\nStuff below.\n",
            BR_START_SENTINEL, BR_END_SENTINEL
        );
        fs::write(&path, &old).unwrap();

        install_br_claude_md_to(tmp.path()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Stuff above."));
        assert!(content.contains("Stuff below."));
        assert!(!content.contains("old content"));
        assert!(content.contains("br ready"));
        assert_eq!(content.matches(BR_START_SENTINEL).count(), 1);
        assert_eq!(content.matches(BR_END_SENTINEL).count(), 1);
    }

    #[test]
    fn idempotent() {
        let tmp = TempDir::new().unwrap();
        install_br_claude_md_to(tmp.path()).unwrap();
        install_br_claude_md_to(tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
        assert_eq!(content.matches(BR_START_SENTINEL).count(), 1);
        assert_eq!(content.matches(BR_END_SENTINEL).count(), 1);
    }

    #[test]
    fn remove_strips_br_section_preserves_rest() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        let content = format!(
            "# My Project\n\nBefore.\n\n{}\n\nAfter.\n",
            BR_GUIDANCE
        );
        fs::write(&path, &content).unwrap();

        remove_br_claude_md_from(tmp.path()).unwrap();

        let result = fs::read_to_string(&path).unwrap();
        assert!(result.contains("# My Project"));
        assert!(result.contains("Before."));
        assert!(result.contains("After."));
        assert!(!result.contains(BR_START_SENTINEL));
        assert!(!result.contains(BR_END_SENTINEL));
    }

    #[test]
    fn remove_noops_when_file_missing() {
        let tmp = TempDir::new().unwrap();
        // No CLAUDE.md file — should succeed silently
        remove_br_claude_md_from(tmp.path()).unwrap();
        assert!(!tmp.path().join("CLAUDE.md").exists());
    }

    #[test]
    fn remove_noops_when_no_br_section() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(&path, "# My Project\n\nNo br here.\n").unwrap();

        remove_br_claude_md_from(tmp.path()).unwrap();

        let result = fs::read_to_string(&path).unwrap();
        assert_eq!(result, "# My Project\n\nNo br here.\n");
    }

    #[test]
    fn br_sentinels_coexist_with_agent_chat_sentinels() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        // Start with agent-chat section already present
        fs::write(&path, "<!-- agent-chat:start -->\n# Agent Chat\n<!-- agent-chat:end -->\n").unwrap();

        install_br_claude_md_to(tmp.path()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("<!-- agent-chat:start -->"));
        assert!(content.contains("<!-- agent-chat:end -->"));
        assert!(content.contains(BR_START_SENTINEL));
        assert!(content.contains(BR_END_SENTINEL));
        assert_eq!(content.matches("<!-- agent-chat:start -->").count(), 1);
        assert_eq!(content.matches(BR_START_SENTINEL).count(), 1);
    }
}
