use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("agent-chat").unwrap()
}

/// Set up a temp dir with .agent-chat/ initialized
fn setup_initialized() -> TempDir {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".agent-chat/log")).unwrap();
    std::fs::create_dir_all(tmp.path().join(".agent-chat/locks")).unwrap();
    std::fs::create_dir_all(tmp.path().join(".agent-chat/cursors")).unwrap();
    std::fs::create_dir_all(tmp.path().join(".agent-chat/sessions")).unwrap();
    std::fs::write(
        tmp.path().join(".agent-chat/config.toml"),
        "lock_ttl_secs = 300\n",
    )
    .unwrap();
    tmp
}

// ── br-claim errors gracefully when br not in PATH ───────────────────

#[test]
fn br_claim_errors_when_br_not_in_path() {
    let tmp = setup_initialized();

    cmd()
        .args(["br-claim", "1"])
        .env("PATH", "/nonexistent")
        .env("AGENT_CHAT_NAME", "test-agent")
        .current_dir(tmp.path())
        .assert()
        .stderr(predicate::str::contains("br (beads_rust) not found"));
}

// ── br-complete errors gracefully when br not in PATH ────────────────

#[test]
fn br_complete_errors_when_br_not_in_path() {
    let tmp = setup_initialized();

    cmd()
        .args(["br-complete", "1"])
        .env("PATH", "/nonexistent")
        .env("AGENT_CHAT_NAME", "test-agent")
        .current_dir(tmp.path())
        .assert()
        .stderr(predicate::str::contains("br (beads_rust) not found"));
}

// ── both fail with "Not initialized" when .agent-chat/ doesn't exist ─

#[test]
fn br_claim_fails_without_agent_chat_dir() {
    let tmp = TempDir::new().unwrap();

    cmd()
        .args(["br-claim", "1"])
        .env("PATH", "/nonexistent")
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not initialized"));
}

#[test]
fn br_complete_fails_without_agent_chat_dir() {
    let tmp = TempDir::new().unwrap();

    cmd()
        .args(["br-complete", "1"])
        .env("PATH", "/nonexistent")
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not initialized"));
}
