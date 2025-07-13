use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::process::Command as StdCommand;

struct AgentGuard {
    name: String,
}

impl AgentGuard {
    fn new(name: &str) -> Self {
        // Clean up any existing agent with this name first
        let _ = StdCommand::new("./target/release/srcrr")
            .args(["rm", name])
            .output();

        Self {
            name: name.to_string(),
        }
    }
}

impl Drop for AgentGuard {
    fn drop(&mut self) {
        // Ensure cleanup happens even if test panics
        let _ = StdCommand::new("./target/release/srcrr")
            .args(["rm", &self.name])
            .output();
    }
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "🧙‍♂️ The Sorcerer - Command agents to do your bidding",
        ))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("rm"))
        .stdout(predicate::str::contains("ps"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("srcrr"))
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_create_missing_name() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("create");

    // With Vec<String>, empty args is valid, should show error message
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("❌ No agent names provided"));
}

#[test]
#[serial]
fn test_list_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📋 Listing agents..."));
}

#[test]
#[serial]
fn test_ps_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("ps");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊 Overview of agents..."));
}

#[test]
#[serial]
fn test_ps_with_lines_option() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["ps", "--lines", "10"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊 Overview of agents..."));
}

#[test]
fn test_ps_invalid_lines_value() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["ps", "--lines", "not-a-number"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_rm_missing_name() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("rm");

    // With Vec<String>, empty args is valid, should show error message
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("❌ No agent names provided"));
}

#[test]
fn test_rm_nonexistent_agent() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "nonexistent-agent"]);

    cmd.assert()
        .success() // Command itself succeeds, but shows error message
        .stdout(predicate::str::contains("💀 Removing agent"))
        .stdout(predicate::str::contains("⚠️  Failed to remove"));
}

#[test]
#[serial]
#[ignore = "Requires working container runtime"]
fn test_create_and_tell_flow() {
    let _guard = AgentGuard::new("test-flow");

    // Summon an agent
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["create", "test-flow"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🌟 Creating agent test-flow"))
        .stdout(predicate::str::contains("✨ Agent test-flow has answered"));

    // List should include our agent
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-flow"));

    // Remove the agent
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "test-flow"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("💀 Removing agent test-flow"))
        .stdout(predicate::str::contains(
            "⚰️  Apprentice test-flow has been removed",
        ));
}

#[test]
fn test_create_command_help() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["create", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Create and start new agent containers",
        ))
        .stdout(predicate::str::contains("NAMES"));
}

#[test]
fn test_list_command_help() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["list", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List all active agents"));
}

#[test]
fn test_rm_command_help() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Stop and remove agent containers"))
        .stdout(predicate::str::contains("NAMES"));
}

#[test]
fn test_ps_command_help() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["ps", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Show detailed status information for all agents",
        ))
        .stdout(predicate::str::contains("--lines"));
}

#[test]
#[serial]
fn test_multiple_subcommands_error() {
    let _guard = AgentGuard::new("ls");

    // Test that we can't use multiple subcommands
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["create", "ls"]);

    // This should treat "ls" as an agent name, not a command
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🌟 Creating agent ls"));
}

#[test]
fn test_ps_with_negative_lines() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["ps", "--lines", "-5"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
#[serial]
fn test_empty_ls_output() {
    // Ensure all agents are removed first
    let _ = StdCommand::new("./target/release/srcrr")
        .args(["rm", "-a"])
        .output();

    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📋 Listing agents"))
        .stdout(predicate::str::contains(
            "The realm is empty - no agents found",
        ));
}

// New tests for multiple agent functionality

#[test]
fn test_create_multiple_agents() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["create", "alice", "bob", "carol"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🌟 Creating agent alice"))
        .stdout(predicate::str::contains("🌟 Creating agent bob"))
        .stdout(predicate::str::contains("🌟 Creating agent carol"))
        .stdout(predicate::str::contains("📊 Summary:"));
}

#[test]
fn test_rm_multiple_agents() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "alice", "bob"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("💀 Removing agent alice"))
        .stdout(predicate::str::contains("💀 Removing agent bob"))
        .stdout(predicate::str::contains("📊 Summary:"));
}

#[test]
#[serial]
fn test_rm_all_flag() {
    // First ensure we have no agents
    let _ = StdCommand::new("./target/release/srcrr")
        .args(["rm", "-a"])
        .output();

    // Test with no agents
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "-a"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📭 No agents to remove"));
}

#[test]
fn test_rm_all_flag_long_form() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "--all"]);

    cmd.assert().success().stdout(
        predicate::str::contains("No agents to remove")
            .or(predicate::str::contains("Removing all")),
    );
}

#[test]
fn test_rm_with_all_flag_and_names_conflict() {
    // The -a flag should take precedence over individual names
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "-a", "alice", "bob"]);

    // This should remove all agents, ignoring the individual names
    cmd.assert().success().stdout(
        predicate::str::contains("No agents to remove")
            .or(predicate::str::contains("Removing all")),
    );
}

#[test]
fn test_create_single_agent() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["create", "single-test"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🌟 Creating agent single-test"))
        // Should NOT show summary for single agent
        .stdout(predicate::str::contains("📊 Summary:").not());
}

#[test]
fn test_rm_single_agent() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "single-test"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("💀 Removing agent single-test"))
        // Should NOT show summary for single agent
        .stdout(predicate::str::contains("📊 Summary:").not());
}

#[test]
#[serial]
#[ignore = "Requires working container runtime"]
fn test_create_and_rm_multiple_flow() {
    // Clean up first
    let _ = StdCommand::new("./target/release/srcrr")
        .args(["rm", "-a"])
        .output();

    // Summon multiple agents
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["create", "test-multi-1", "test-multi-2", "test-multi-3"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊 Summary:"));

    // List should show all three
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-multi-1"))
        .stdout(predicate::str::contains("test-multi-2"))
        .stdout(predicate::str::contains("test-multi-3"));

    // Remove two of them
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "test-multi-1", "test-multi-2"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊 Summary: 2/2"));

    // List should only show one
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-multi-3"))
        .stdout(predicate::str::contains("test-multi-1").not())
        .stdout(predicate::str::contains("test-multi-2").not());

    // Remove all remaining
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "-a"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🗑️  Removing all 1 agents"));
}
