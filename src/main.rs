mod cli;
mod commands;
mod error;
mod format;
mod hooks;
mod names;
mod storage;

use clap::Parser;
use cli::{Cli, Command};
use std::process;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Init { project, user, both } => {
            let cwd = std::env::current_dir().unwrap_or_else(|e| {
                eprintln!("Cannot determine current directory: {}", e);
                process::exit(1);
            });
            commands::init::run(&cwd, project, user, both)
        }
        Command::Register => {
            let root = find_root_or_exit();
            commands::register::run(&root)
        }
        Command::Say { message } => {
            let root = find_root_or_exit();
            let msg = message.join(" ");
            if msg.is_empty() {
                eprintln!("Message cannot be empty.");
                process::exit(1);
            }
            commands::say::run(&root, &msg)
        }
        Command::Read { all } => {
            let root = find_root_or_exit();
            commands::read::run(&root, all)
        }
        Command::Status => {
            let root = find_root_or_exit();
            commands::status::run(&root)
        }
        Command::Lock { glob } => {
            let root = find_root_or_exit();
            commands::lock::acquire(&root, &glob)
        }
        Command::Unlock { glob } => {
            let root = find_root_or_exit();
            commands::lock::release(&root, &glob)
        }
        Command::Locks => {
            let root = find_root_or_exit();
            commands::lock::list(&root)
        }
        Command::CheckLock => {
            let root = find_root_or_exit();
            commands::check_lock::run(&root)
        }
        Command::CheckMessages => {
            let root = find_root_or_exit();
            commands::check_messages::run(&root)
        }
        Command::InitBr { project, user } => {
            let cwd = std::env::current_dir().unwrap_or_else(|e| {
                eprintln!("Cannot determine current directory: {}", e);
                process::exit(1);
            });
            commands::init_br::run(&cwd, project, user)
        }
        Command::BrClaim { id } => {
            let root = find_root_or_exit();
            commands::br_claim::run(&root, &id)
        }
        Command::BrComplete { id, reason } => {
            let root = find_root_or_exit();
            commands::br_complete::run(&root, &id, reason.as_deref())
        }
    };

    if let Err(e) = result {
        // Hook commands exit 0 even on error (advisory, never block)
        eprintln!("{}", e);
        process::exit(0);
    }
}

fn find_root_or_exit() -> std::path::PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("Cannot determine current directory: {}", e);
        process::exit(1);
    });
    match storage::paths::find_root(&cwd) {
        Ok(root) => root,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
