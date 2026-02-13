use std::io::{self, BufRead, Write};
use std::path::Path;
use crate::error::{AgentChatError, Result};
use crate::hooks::agents_md_codex;
use crate::storage::{config, paths};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexInstallTarget {
    Project,
    User,
    Both,
}

fn resolve_target(project: bool, user: bool, both: bool) -> Result<CodexInstallTarget> {
    if both {
        return Ok(CodexInstallTarget::Both);
    }
    if project && user {
        return Ok(CodexInstallTarget::Both);
    }
    if project {
        return Ok(CodexInstallTarget::Project);
    }
    if user {
        return Ok(CodexInstallTarget::User);
    }

    // Interactive prompt
    eprint!(
        "Where should Codex AGENTS.md guidance be installed?\n\
         \x20 1. Project  — ./AGENTS.md\n\
         \x20 2. User     — ~/.codex/AGENTS.md\n\
         \x20 3. Both\n\
         > "
    );
    io::stderr().flush()?;

    let stdin = io::stdin();
    let line = stdin.lock().lines().next()
        .unwrap_or(Err(io::Error::new(io::ErrorKind::UnexpectedEof, "no input")))?;

    match line.trim() {
        "1" => Ok(CodexInstallTarget::Project),
        "2" => Ok(CodexInstallTarget::User),
        "3" => Ok(CodexInstallTarget::Both),
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
        CodexInstallTarget::Project => {
            agents_md_codex::install_agents_md_to(project_root)?;
            println!("Initialized .agent-chat/ and installed Codex guidance (project).");
        }
        CodexInstallTarget::User => {
            let home = paths::home_dir()?;
            let codex_dir = home.join(".codex");
            agents_md_codex::install_agents_md_to(&codex_dir)?;
            paths::add_git_exclude(project_root, ".agent-chat/")?;
            println!("Initialized .agent-chat/ and installed Codex guidance (user).");
        }
        CodexInstallTarget::Both => {
            agents_md_codex::install_agents_md_to(project_root)?;
            let home = paths::home_dir()?;
            let codex_dir = home.join(".codex");
            agents_md_codex::install_agents_md_to(&codex_dir)?;
            paths::add_git_exclude(project_root, ".agent-chat/")?;
            println!("Initialized .agent-chat/ and installed Codex guidance (project + user).");
        }
    }

    Ok(())
}
