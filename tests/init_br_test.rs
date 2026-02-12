use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("agent-chat").unwrap()
}

// ── --project ────────────────────────────────────────────────────────

#[test]
fn init_br_project_installs_br_section() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    cmd()
        .args(["init-br", "--project"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project"));

    let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("<!-- agent-chat-br:start -->"));
    assert!(content.contains("<!-- agent-chat-br:end -->"));
    assert!(content.contains("br ready"));
}

#[test]
fn init_br_project_does_not_touch_user_claude_md() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    cmd()
        .args(["init-br", "--project"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(!fake_home.path().join(".claude/CLAUDE.md").exists());
}

// ── --user ───────────────────────────────────────────────────────────

#[test]
fn init_br_user_installs_to_user_claude_md() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    cmd()
        .args(["init-br", "--user"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("user"));

    let content = std::fs::read_to_string(fake_home.path().join(".claude/CLAUDE.md")).unwrap();
    assert!(content.contains("<!-- agent-chat-br:start -->"));
    assert!(content.contains("br ready"));

    // No project CLAUDE.md
    assert!(!tmp.path().join("CLAUDE.md").exists());
}

// ── auto-cleanup ─────────────────────────────────────────────────────

#[test]
fn init_br_project_removes_br_from_user() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    // First install to user
    cmd()
        .args(["init-br", "--user"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();
    assert!(fake_home.path().join(".claude/CLAUDE.md").exists());

    // Now switch to project — should remove from user
    cmd()
        .args(["init-br", "--project"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    // Project has br section
    let project_content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(project_content.contains("<!-- agent-chat-br:start -->"));

    // User no longer has br section (file removed since it was only br content)
    assert!(!fake_home.path().join(".claude/CLAUDE.md").exists());
}

#[test]
fn init_br_user_removes_br_from_project() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    // First install to project
    cmd()
        .args(["init-br", "--project"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();
    assert!(tmp.path().join("CLAUDE.md").exists());

    // Now switch to user — should remove from project
    cmd()
        .args(["init-br", "--user"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    // User has br section
    let user_content = std::fs::read_to_string(fake_home.path().join(".claude/CLAUDE.md")).unwrap();
    assert!(user_content.contains("<!-- agent-chat-br:start -->"));

    // Project no longer has br section (file removed since it was only br content)
    assert!(!tmp.path().join("CLAUDE.md").exists());
}

// ── idempotent ───────────────────────────────────────────────────────

#[test]
fn init_br_project_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    cmd()
        .args(["init-br", "--project"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();
    cmd()
        .args(["init-br", "--project"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert_eq!(content.matches("<!-- agent-chat-br:start -->").count(), 1);
    assert_eq!(content.matches("<!-- agent-chat-br:end -->").count(), 1);
}

// ── coexistence with agent-chat section ──────────────────────────────

#[test]
fn br_section_coexists_with_agent_chat_section() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    // First run agent-chat init to get agent-chat section
    cmd()
        .args(["init", "--project"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    // Then add br section
    cmd()
        .args(["init-br", "--project"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("<!-- agent-chat:start -->"));
    assert!(content.contains("<!-- agent-chat:end -->"));
    assert!(content.contains("<!-- agent-chat-br:start -->"));
    assert!(content.contains("<!-- agent-chat-br:end -->"));
    assert_eq!(content.matches("<!-- agent-chat:start -->").count(), 1);
    assert_eq!(content.matches("<!-- agent-chat-br:start -->").count(), 1);
}

// ── --project --user together → error ────────────────────────────────

#[test]
fn init_br_project_and_user_together_errors() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    cmd()
        .args(["init-br", "--project", "--user"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .stderr(predicate::str::contains("Cannot specify both"));
}

// ── no flags without stdin shows prompt ──────────────────────────────

#[test]
fn init_br_no_flags_no_stdin_shows_prompt() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    cmd()
        .args(["init-br"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .stderr(predicate::str::contains("Where should br guidance"))
        .stderr(predicate::str::contains("no input"));
}
