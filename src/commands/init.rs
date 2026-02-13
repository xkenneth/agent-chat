use std::io::{self, BufRead, Write};
use std::path::Path;
use crate::error::{AgentChatError, Result};
use crate::storage::{config, paths};
use crate::hooks::{agents_md_codex, claude_md, installer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallTarget {
    Project,
    User,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolTarget {
    Claude,
    Codex,
    Both,
}

fn resolve_target(project: bool, user: bool, both: bool, tool_target: ToolTarget) -> Result<InstallTarget> {
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
    match tool_target {
        ToolTarget::Claude => {
            eprint!(
                "\nInstall target for Claude integration:\n\
                 \x20 [1] Project  -> .claude/settings.local.json + ./CLAUDE.md\n\
                 \x20 [2] User     -> ~/.claude/settings.json + ~/.claude/CLAUDE.md (default)\n\
                 \x20 [3] Both\n\
                 Select 1/2/3 (Enter = default) > "
            );
        }
        ToolTarget::Codex => {
            eprint!(
                "\nInstall target for Codex integration:\n\
                 \x20 [1] Project  -> ./AGENTS.md\n\
                 \x20 [2] User     -> ~/.codex/AGENTS.md (default)\n\
                 \x20 [3] Both\n\
                 Select 1/2/3 (Enter = default) > "
            );
        }
        ToolTarget::Both => {
            eprint!(
                "\nInstall target for Claude + Codex integrations:\n\
                 \x20 [1] Project  -> .claude/settings.local.json + ./CLAUDE.md + ./AGENTS.md\n\
                 \x20 [2] User     -> ~/.claude/settings.json + ~/.claude/CLAUDE.md + ~/.codex/AGENTS.md (default)\n\
                 \x20 [3] Both\n\
                 Select 1/2/3 (Enter = default) > "
            );
        }
    }
    io::stderr().flush()?;

    let stdin = io::stdin();
    let line = stdin.lock().lines().next()
        .unwrap_or(Err(io::Error::new(io::ErrorKind::UnexpectedEof, "no input")))?;

    match line.trim() {
        "1" => Ok(InstallTarget::Project),
        "" | "2" => Ok(InstallTarget::User),
        "3" => Ok(InstallTarget::Both),
        other => Err(AgentChatError::Other(format!("invalid choice: {}", other))),
    }
}

fn resolve_tools(
    claude: bool,
    codex: bool,
    both_tools: bool,
    has_location_flags: bool,
) -> Result<ToolTarget> {
    if both_tools || (claude && codex) {
        return Ok(ToolTarget::Both);
    }
    if claude {
        return Ok(ToolTarget::Claude);
    }
    if codex {
        return Ok(ToolTarget::Codex);
    }

    // Backward compatibility for non-interactive scripts like `init --project`.
    if has_location_flags {
        return Ok(ToolTarget::Claude);
    }

    eprint!(
        "\nAgent Chat setup\n\
         Choose integration(s):\n\
         \x20 [1] Claude (hooks + CLAUDE.md)\n\
         \x20 [2] Codex  (AGENTS.md)\n\
         \x20 [3] Both (default)\n\
         Select 1/2/3 (Enter = default) > "
    );
    io::stderr().flush()?;

    let stdin = io::stdin();
    let line = stdin.lock().lines().next()
        .unwrap_or(Err(io::Error::new(io::ErrorKind::UnexpectedEof, "no input")))?;

    match line.trim() {
        "1" => Ok(ToolTarget::Claude),
        "2" => Ok(ToolTarget::Codex),
        "" | "3" => Ok(ToolTarget::Both),
        other => Err(AgentChatError::Other(format!("invalid choice: {}", other))),
    }
}

pub fn run(
    project_root: &Path,
    project: bool,
    user: bool,
    both: bool,
    claude: bool,
    codex: bool,
    both_tools: bool,
) -> Result<()> {
    let has_location_flags = project || user || both;
    let tool_target = resolve_tools(claude, codex, both_tools, has_location_flags)?;
    let target = resolve_target(project, user, both, tool_target)?;

    // Always create .agent-chat/ + config in the project
    paths::create_dirs(project_root)?;
    let root = project_root.join(".agent-chat");
    let config_path = paths::config_path(&root);
    if !config_path.exists() {
        config::write_default_config(&config_path)?;
    }

    match (tool_target, target) {
        (ToolTarget::Claude, InstallTarget::Project) => {
            installer::install_hooks(project_root)?;
            claude_md::install_claude_md(project_root)?;
            println!("Initialized .agent-chat/ and installed hooks (project).");
        }
        (ToolTarget::Claude, InstallTarget::User) => {
            let home = paths::home_dir()?;
            let claude_dir = home.join(".claude");
            installer::install_hooks_to(&claude_dir, "settings.json")?;
            claude_md::install_claude_md_to(&claude_dir)?;
            paths::add_git_exclude(project_root, ".agent-chat/")?;
            println!("Initialized .agent-chat/ and installed hooks (user).");
        }
        (ToolTarget::Claude, InstallTarget::Both) => {
            installer::install_hooks(project_root)?;
            claude_md::install_claude_md(project_root)?;
            let home = paths::home_dir()?;
            let claude_dir = home.join(".claude");
            installer::install_hooks_to(&claude_dir, "settings.json")?;
            claude_md::install_claude_md_to(&claude_dir)?;
            paths::add_git_exclude(project_root, ".agent-chat/")?;
            println!("Initialized .agent-chat/ and installed hooks (project + user).");
        }
        (ToolTarget::Codex, InstallTarget::Project) => {
            agents_md_codex::install_agents_md_to(project_root)?;
            println!("Initialized .agent-chat/ and installed Codex guidance (project).");
        }
        (ToolTarget::Codex, InstallTarget::User) => {
            let home = paths::home_dir()?;
            let codex_dir = home.join(".codex");
            agents_md_codex::install_agents_md_to(&codex_dir)?;
            paths::add_git_exclude(project_root, ".agent-chat/")?;
            println!("Initialized .agent-chat/ and installed Codex guidance (user).");
        }
        (ToolTarget::Codex, InstallTarget::Both) => {
            agents_md_codex::install_agents_md_to(project_root)?;
            let home = paths::home_dir()?;
            let codex_dir = home.join(".codex");
            agents_md_codex::install_agents_md_to(&codex_dir)?;
            paths::add_git_exclude(project_root, ".agent-chat/")?;
            println!("Initialized .agent-chat/ and installed Codex guidance (project + user).");
        }
        (ToolTarget::Both, InstallTarget::Project) => {
            installer::install_hooks(project_root)?;
            claude_md::install_claude_md(project_root)?;
            agents_md_codex::install_agents_md_to(project_root)?;
            println!("Initialized .agent-chat/ and installed Claude + Codex integrations (project).");
        }
        (ToolTarget::Both, InstallTarget::User) => {
            let home = paths::home_dir()?;
            let claude_dir = home.join(".claude");
            let codex_dir = home.join(".codex");
            installer::install_hooks_to(&claude_dir, "settings.json")?;
            claude_md::install_claude_md_to(&claude_dir)?;
            agents_md_codex::install_agents_md_to(&codex_dir)?;
            paths::add_git_exclude(project_root, ".agent-chat/")?;
            println!("Initialized .agent-chat/ and installed Claude + Codex integrations (user).");
        }
        (ToolTarget::Both, InstallTarget::Both) => {
            installer::install_hooks(project_root)?;
            claude_md::install_claude_md(project_root)?;
            agents_md_codex::install_agents_md_to(project_root)?;
            let home = paths::home_dir()?;
            let claude_dir = home.join(".claude");
            let codex_dir = home.join(".codex");
            installer::install_hooks_to(&claude_dir, "settings.json")?;
            claude_md::install_claude_md_to(&claude_dir)?;
            agents_md_codex::install_agents_md_to(&codex_dir)?;
            paths::add_git_exclude(project_root, ".agent-chat/")?;
            println!("Initialized .agent-chat/ and installed Claude + Codex integrations (project + user).");
        }
    }

    Ok(())
}
