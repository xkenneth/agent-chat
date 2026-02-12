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
fn check_messages_ignores_own() {
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

    // Same agent runs check-messages — should produce no output (own message filtered)
    cmd()
        .arg("check-messages")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn check_messages_shows_others() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // A posts a message
    cmd()
        .args(["say", "hello from A"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // B runs check-messages — should see additionalContext JSON with message content
    let output = cmd()
        .arg("check-messages")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("additionalContext"), "Expected additionalContext JSON but got: {}", stdout);
    assert!(stdout.contains("1 unread message"), "Expected unread count but got: {}", stdout);
    assert!(stdout.contains("hello from A"), "Expected message content but got: {}", stdout);
    assert!(stdout.contains("swift-fox"), "Expected sender name but got: {}", stdout);
}

#[test]
fn check_messages_advances_cursor() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // A posts a message
    cmd()
        .args(["say", "hello from A"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // B runs check-messages — should see the message
    cmd()
        .arg("check-messages")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello from A"));

    std::thread::sleep(std::time::Duration::from_millis(50));

    // B runs check-messages again — cursor was advanced, should be empty now
    cmd()
        .arg("check-messages")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn check_messages_includes_message_content() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // A posts two messages
    cmd()
        .args(["say", "first message"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();
    std::thread::sleep(std::time::Duration::from_millis(10));
    cmd()
        .args(["say", "second message"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // B runs check-messages — should see both messages formatted
    let output = cmd()
        .arg("check-messages")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse the JSON to check structure
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect(&format!("Expected valid JSON but got: {}", stdout));

    let context = json["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("Expected additionalContext string");

    assert!(context.contains("2 unread messages"), "Expected '2 unread messages' in: {}", context);
    assert!(context.contains("first message"), "Expected 'first message' in: {}", context);
    assert!(context.contains("second message"), "Expected 'second message' in: {}", context);
    assert!(context.contains("swift-fox"), "Expected 'swift-fox' in: {}", context);
}
