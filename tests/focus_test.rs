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
fn focus_set_and_list() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["focus", "CI pipeline"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Focus set: CI pipeline"));

    cmd()
        .arg("focuses")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("swift-fox"))
        .stdout(predicate::str::contains("CI pipeline"));
}

#[test]
fn focus_clear() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["focus", "CI pipeline"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    cmd()
        .args(["focus", "--clear"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Focus cleared"));

    cmd()
        .arg("focuses")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No active focuses"));
}

#[test]
fn focus_replaces_previous() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["focus", "CI pipeline"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    cmd()
        .args(["focus", "API work"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    cmd()
        .arg("focuses")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("API work"))
        .stdout(predicate::str::contains("CI pipeline").not());
}

#[test]
fn multiple_agents_focuses() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["focus", "CI pipeline"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    cmd()
        .args(["focus", "API work"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success();

    cmd()
        .arg("focuses")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("swift-fox"))
        .stdout(predicate::str::contains("bold-hawk"));
}

#[test]
fn focus_no_args_errors() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .arg("focus")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .failure();
}

#[test]
fn focuses_empty() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .arg("focuses")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No active focuses"));
}
