use std::io::{self, BufRead, Write};
use std::path::Path;
use crate::error::{AgentChatError, Result};
use crate::storage::{config, paths};
use crate::hooks::{claude_md, installer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallTarget {
    Project,
    User,
    Both,
}

fn resolve_target(project: bool, user: bool, both: bool) -> Result<InstallTarget> {
    if both {
        return Ok(InstallTarget::Both);
    }
    if project && user {
        return Ok(InstallTarget::Both);
    }
    if project {
        return Ok(InstallTarget::Project);
    }
    if user {
        return Ok(InstallTarget::User);
    }

    // Interactive prompt
    eprint!(
        "Where should hooks and CLAUDE.md be installed?\n\
         \x20 1. Project  — .claude/settings.local.json + ./CLAUDE.md\n\
         \x20 2. User     — ~/.claude/settings.json + ~/.claude/CLAUDE.md\n\
         \x20 3. Both\n\
         > "
    );
    io::stderr().flush()?;

    let stdin = io::stdin();
    let line = stdin.lock().lines().next()
        .unwrap_or(Err(io::Error::new(io::ErrorKind::UnexpectedEof, "no input")))?;

    match line.trim() {
        "1" => Ok(InstallTarget::Project),
        "2" => Ok(InstallTarget::User),
        "3" => Ok(InstallTarget::Both),
        other => Err(AgentChatError::Other(format!("invalid choice: {}", other))),
    }
}

pub fn run(project_root: &Path, project: bool, user: bool, both: bool) -> Result<()> {
    let target = resolve_target(project, user, both)?;

    // Always create .agent-chat/ + config in the project
    paths::create_dirs(project_root)?;
    let root = project_root.join(".agent-chat");
    let config_path = paths::config_path(&root);
    if !config_path.exists() {
        config::write_default_config(&config_path)?;
    }

    match target {
        InstallTarget::Project => {
            installer::install_hooks(project_root)?;
            claude_md::install_claude_md(project_root)?;
            println!("Initialized .agent-chat/ and installed hooks (project).");
        }
        InstallTarget::User => {
            let home = paths::home_dir()?;
            let claude_dir = home.join(".claude");
            installer::install_hooks_to(&claude_dir, "settings.json")?;
            claude_md::install_claude_md_to(&claude_dir)?;
            paths::add_git_exclude(project_root, ".agent-chat/")?;
            println!("Initialized .agent-chat/ and installed hooks (user).");
        }
        InstallTarget::Both => {
            installer::install_hooks(project_root)?;
            claude_md::install_claude_md(project_root)?;
            let home = paths::home_dir()?;
            let claude_dir = home.join(".claude");
            installer::install_hooks_to(&claude_dir, "settings.json")?;
            claude_md::install_claude_md_to(&claude_dir)?;
            paths::add_git_exclude(project_root, ".agent-chat/")?;
            println!("Initialized .agent-chat/ and installed hooks (project + user).");
        }
    }

    Ok(())
}
