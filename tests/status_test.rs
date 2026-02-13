use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("agent-chat").unwrap()
}

fn init_project(tmp: &TempDir) {
    cmd().args(["init", "--project"]).current_dir(tmp.path()).assert().success();
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

    // Check status from another session — should see decision:block JSON
    let output = cmd()
        .arg("status")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect(&format!("Expected valid JSON but got: {}", stdout));

    assert_eq!(json["decision"], "block", "Expected decision:block but got: {}", stdout);
    let reason = json["reason"].as_str().expect("Expected reason string");
    assert!(reason.contains("[agent-chat:"), "Expected agent-chat header in reason: {}", reason);
    assert!(reason.contains("hello"), "Expected message content in reason: {}", reason);
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
fn status_ignores_own_messages() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Agent posts a message
    cmd()
        .args(["say", "my own message"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // Same agent checks status — should see nothing (own message filtered)
    cmd()
        .arg("status")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn status_counts_only_others() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // A posts 2 messages
    cmd()
        .args(["say", "msg 1"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();
    std::thread::sleep(std::time::Duration::from_millis(10));
    cmd()
        .args(["say", "msg 2"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();
    std::thread::sleep(std::time::Duration::from_millis(10));

    // B posts 1 message
    cmd()
        .args(["say", "from B"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success();

    // A checks status — should see decision:block with only B's message
    let output = cmd()
        .arg("status")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect(&format!("Expected valid JSON but got: {}", stdout));

    assert_eq!(json["decision"], "block");
    let reason = json["reason"].as_str().expect("Expected reason string");
    assert!(reason.contains("1 unread message"), "Expected '1 unread message' but got: {}", reason);
    assert!(reason.contains("from B"), "Expected 'from B' but got: {}", reason);
    assert!(reason.contains("bold-hawk"), "Expected 'bold-hawk' but got: {}", reason);
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

#[test]
fn status_noop_when_identity_missing() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Create messages from another agent.
    cmd()
        .args(["say", "hello"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // No AGENT_CHAT_* env and multiple sessions -> unresolved identity.
    // Stop hook should not block in this case.
    cmd()
        .arg("status")
        .current_dir(tmp.path())
        .env_remove("AGENT_CHAT_NAME")
        .env_remove("AGENT_CHAT_SESSION_ID")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
