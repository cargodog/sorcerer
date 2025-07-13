use serial_test::serial;
use std::process::Command;
use std::time::Duration;

struct ApprenticeGuard {
    name: String,
}

impl ApprenticeGuard {
    fn new(name: &str) -> Self {
        // Clean up any existing agent with this name first
        let _ = Command::new("./target/release/srcrr")
            .args(["rm", name])
            .output();

        // Wait for cleanup to complete
        std::thread::sleep(Duration::from_millis(500));

        Self {
            name: name.to_string(),
        }
    }
}

impl Drop for ApprenticeGuard {
    fn drop(&mut self) {
        // Ensure cleanup happens even if test panics
        let _ = Command::new("./target/release/srcrr")
            .args(["rm", &self.name])
            .output();
    }
}

/// Clean up all common test agents
/// This can be called manually to ensure no test agents are left running
#[allow(dead_code)]
fn cleanup_all_test_agents() {
    let test_names = [
        "container-test",
        "duplicate-test",
        "test-agent",
        "test_formatting",
    ];

    for name in &test_names {
        let _ = Command::new("./target/release/srcrr")
            .args(["rm", name])
            .output();
    }

    // Wait for all cleanups to complete
    std::thread::sleep(Duration::from_millis(500));
}

#[test]
#[serial]
// Requires working container runtime
fn test_create_and_list() {
    // Use guard to ensure cleanup even if test panics
    let _guard = ApprenticeGuard::new("container-test");

    // Test createing
    let output = Command::new("./target/release/srcrr")
        .args(["create", "container-test"])
        .output()
        .expect("Failed to execute create command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Summon failed. stdout: {stdout}, stderr: {stderr}"
    );
    assert!(
        stdout.contains("has answered your call"),
        "Unexpected create output: {stdout}"
    );

    // Wait for container to be ready
    std::thread::sleep(Duration::from_secs(3));

    // Test list includes our agent
    let output = Command::new("./target/release/srcrr")
        .arg("list")
        .output()
        .expect("Failed to execute list command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("container-test"),
        "Ls should show our agent"
    );

    // Cleanup handled automatically by ApprenticeGuard
}

#[test]
#[serial]
fn test_create_duplicate_failist() {
    // This test doesn't require API key or containers
    // First create might fail if no runtime, but duplicate check should still work

    let name = "duplicate-test";
    let _guard = ApprenticeGuard::new(name);

    // Try to create twice
    let _ = Command::new("./target/release/srcrr")
        .args(["create", name])
        .output();

    // Second create should fail with "already exists" if first succeeded
    let output = Command::new("./target/release/srcrr")
        .args(["create", name])
        .output()
        .expect("Failed to execute create command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If container runtime is available, it should say "already exists"
    // If not available, it will fail with a different error
    // This at least tests the CLI is working
    assert!(
        stdout.contains("already exists") || stdout.contains("Failed to create"),
        "Unexpected output: {stdout}"
    );

    // Cleanup handled automatically by ApprenticeGuard
}

#[test]
fn test_invalid_agent_name() {
    let output = Command::new("./target/release/srcrr")
        .args(["create", "invalid name with spaces"])
        .output()
        .expect("Failed to execute create command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Invalid agent name") || stdout.contains("Failed to create"),
        "Should reject invalid name"
    );
}
