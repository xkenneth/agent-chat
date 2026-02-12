use std::path::Path;
use crate::error::Result;
use crate::storage::{config, paths};
use crate::hooks::{claude_md, installer};

pub fn run(project_root: &Path) -> Result<()> {
    paths::create_dirs(project_root)?;

    let root = project_root.join(".agent-chat");
    let config_path = paths::config_path(&root);
    if !config_path.exists() {
        config::write_default_config(&config_path)?;
    }

    installer::install_hooks(project_root)?;
    claude_md::install_claude_md(project_root)?;

    println!("Initialized .agent-chat/ and installed Claude Code hooks.");
    Ok(())
}
