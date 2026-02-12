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
fn status_nothing_when_empty() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .arg("status")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn status_shows_unread_when_new_messages() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Post a message
    cmd()
        .args(["say", "hello"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // Check status from another session
    cmd()
        .arg("status")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success()
        .stdout(predicate::str::contains("[agent-chat:"));
}

#[test]
fn status_nothing_after_read() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Post a message
    cmd()
        .args(["say", "hello"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // Read from sess2
    cmd()
        .arg("read")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success();

    std::thread::sleep(std::time::Duration::from_millis(50));

    // Status should now be empty
    cmd()
        .arg("status")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn status_performance() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    let start = std::time::Instant::now();
    cmd()
        .arg("status")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "perf-test")
        .assert()
        .success();
    let elapsed = start.elapsed();

    // Should complete in well under 100ms (target is <10ms)
    assert!(
        elapsed.as_millis() < 100,
        "Status took {}ms, expected <100ms",
        elapsed.as_millis()
    );
}
