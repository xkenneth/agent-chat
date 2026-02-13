use std::path::Path;
use crate::error::Result;
use crate::storage::{config, focus as focus_store, identity, paths};
use crate::ui;

pub fn set(root: &Path, text: &str) -> Result<()> {
    let id = identity::resolve(root)?;
    let name = identity::require_name(&id)?;

    let config = config::read_config(&paths::config_path(root))?;
    let focuses_dir = paths::focuses_dir(root);

    focus_store::set(&focuses_dir, text, name, &id.session_id, config.focus_ttl_secs)?;
    println!("{}", ui::success_line("Focus set:", text));
    Ok(())
}

pub fn clear(root: &Path) -> Result<()> {
    let id = identity::resolve(root)?;

    let focuses_dir = paths::focuses_dir(root);
    focus_store::clear(&focuses_dir, &id.session_id)?;
    println!("{}", ui::success_line("Focus cleared.", ""));
    Ok(())
}

pub fn list(root: &Path) -> Result<()> {
    let focuses_dir = paths::focuses_dir(root);
    let focuses = focus_store::list_active(&focuses_dir)?;

    if focuses.is_empty() {
        println!("{}", ui::info_line("Focuses:", "No active focuses."));
        return Ok(());
    }

    println!("{}", ui::table_header("AGENT", "FOCUS", None));
    for f in &focuses {
        println!("{:<15} {}", f.owner, f.focus);
    }
    Ok(())
}
