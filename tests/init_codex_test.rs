use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("agent-chat").unwrap()
}

#[test]
fn init_codex_project_creates_agents_md_and_not_claude_files() {
    let tmp = TempDir::new().unwrap();

    cmd()
        .args(["init-codex", "--project"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Codex guidance"));

    assert!(tmp.path().join(".agent-chat/log").is_dir());
    assert!(tmp.path().join(".agent-chat/config.toml").exists());
    assert!(tmp.path().join("AGENTS.md").exists());

    let content = std::fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(content.contains("<!-- agent-chat-codex:start -->"));
    assert!(content.contains("agent-chat register --session-id"));

    // Codex init should not install Claude hooks/docs.
    assert!(!tmp.path().join(".claude/settings.local.json").exists());
    assert!(!tmp.path().join("CLAUDE.md").exists());
}

#[test]
fn init_codex_user_creates_user_agents_md() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join(".git")).unwrap();

    cmd()
        .args(["init-codex", "--user"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(fake_home.path().join(".codex/AGENTS.md").exists());
    assert!(!fake_home.path().join(".claude/settings.json").exists());
    assert!(!fake_home.path().join(".claude/CLAUDE.md").exists());

    let exclude = std::fs::read_to_string(tmp.path().join(".git/info/exclude")).unwrap();
    assert!(exclude.contains(".agent-chat/"));
}

#[test]
fn init_codex_is_idempotent() {
    let tmp = TempDir::new().unwrap();

    cmd().args(["init-codex", "--project"]).current_dir(tmp.path()).assert().success();
    cmd().args(["init-codex", "--project"]).current_dir(tmp.path()).assert().success();

    let content = std::fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert_eq!(content.matches("<!-- agent-chat-codex:start -->").count(), 1);
    assert_eq!(content.matches("<!-- agent-chat-codex:end -->").count(), 1);
}

#[test]
fn init_codex_preserves_existing_agents_md() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("AGENTS.md"), "# Existing\n\nKeep me.\n").unwrap();

    cmd().args(["init-codex", "--project"]).current_dir(tmp.path()).assert().success();

    let content = std::fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(content.contains("# Existing"));
    assert!(content.contains("Keep me."));
    assert!(content.contains("<!-- agent-chat-codex:start -->"));
}
