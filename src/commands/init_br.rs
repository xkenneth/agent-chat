use std::io::{self, BufRead, Write};
use std::path::Path;
use crate::error::{AgentChatError, Result};
use crate::hooks::claude_md_br;
use crate::storage::paths;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrInstallTarget {
    Project,
    User,
}

fn resolve_target(project: bool, user: bool) -> Result<BrInstallTarget> {
    if project && user {
        return Err(AgentChatError::Other(
            "Cannot specify both --project and --user. Choose one.".to_string()
        ));
    }
    if project {
        return Ok(BrInstallTarget::Project);
    }
    if user {
        return Ok(BrInstallTarget::User);
    }

    // Interactive prompt
    eprint!(
        "Where should br guidance be installed?\n\
         \x20 1. Project  — ./CLAUDE.md\n\
         \x20 2. User     — ~/.claude/CLAUDE.md\n\
         > "
    );
    io::stderr().flush()?;

    let stdin = io::stdin();
    let line = stdin.lock().lines().next()
        .unwrap_or(Err(io::Error::new(io::ErrorKind::UnexpectedEof, "no input")))?;

    match line.trim() {
        "1" => Ok(BrInstallTarget::Project),
        "2" => Ok(BrInstallTarget::User),
        other => Err(AgentChatError::Other(format!("invalid choice: {}", other))),
    }
}

pub fn run(project_root: &Path, project: bool, user: bool) -> Result<()> {
    let target = resolve_target(project, user)?;

    match target {
        BrInstallTarget::Project => {
            claude_md_br::install_br_claude_md_to(project_root)?;
            // Auto-cleanup: remove from user level
            let home = paths::home_dir()?;
            let claude_dir = home.join(".claude");
            claude_md_br::remove_br_claude_md_from(&claude_dir)?;
            println!("Installed br guidance (project).");
        }
        BrInstallTarget::User => {
            let home = paths::home_dir()?;
            let claude_dir = home.join(".claude");
            claude_md_br::install_br_claude_md_to(&claude_dir)?;
            // Auto-cleanup: remove from project level
            claude_md_br::remove_br_claude_md_from(project_root)?;
            println!("Installed br guidance (user).");
        }
    }

    Ok(())
}
