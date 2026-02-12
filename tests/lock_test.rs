use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("agent-chat").unwrap()
}

fn init_project(tmp: &TempDir) {
    cmd().arg("init").current_dir(tmp.path()).assert().success();
}

#[test]
fn lock_creates_lockfile() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["lock", "src/*.rs"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Locked: src/*.rs"));

    // Should appear in locks list
    cmd()
        .arg("locks")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/*.rs"))
        .stdout(predicate::str::contains("swift-fox"));
}

#[test]
fn unlock_removes_lockfile() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["lock", "src/*.rs"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    cmd()
        .args(["unlock", "src/*.rs"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Unlocked: src/*.rs"));

    cmd()
        .arg("locks")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::contains("No active locks"));
}

#[test]
fn lock_conflict_errors() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["lock", "src/*.rs"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // Different session trying to lock same pattern
    cmd()
        .args(["lock", "src/*.rs"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success() // exits 0 (advisory)
        .stderr(predicate::str::contains("Lock conflict"));
}

#[test]
fn different_patterns_ok() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["lock", "src/*.rs"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    cmd()
        .args(["lock", "tests/*.rs"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Locked: tests/*.rs"));
}
