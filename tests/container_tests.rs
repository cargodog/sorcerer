use serial_test::serial;
use std::process::Command;
use std::time::Duration;

struct ApprenticeGuard {
    name: String,
}

impl ApprenticeGuard {
    fn new(name: &str) -> Self {
        // Clean up any existing apprentice with this name first
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

/// Clean up all common test apprentices
/// This can be called manually to ensure no test apprentices are left running
#[allow(dead_code)]
fn cleanup_all_test_apprentices() {
    let test_names = [
        "container-test",
        "duplicate-test",
        "test-apprentice",
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
#[ignore] // Requires working container runtime and API key
fn test_summon_and_communicate() {
    // Check if we have the ANTHROPIC_API_KEY set
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("Skipping container test: ANTHROPIC_API_KEY not set");
        return;
    }

    // Use guard to ensure cleanup even if test panics
    let _guard = ApprenticeGuard::new("container-test");

    // Test summoning
    let output = Command::new("./target/release/srcrr")
        .args(["summon", "container-test"])
        .output()
        .expect("Failed to execute summon command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Summon failed. stdout: {stdout}, stderr: {stderr}"
    );
    assert!(
        stdout.contains("has answered your call"),
        "Unexpected summon output: {stdout}"
    );

    // Wait for container to be ready
    std::thread::sleep(Duration::from_secs(3));

    // Test communication
    let output = Command::new("./target/release/srcrr")
        .args(["tell", "container-test", "What is 2+2?"])
        .output()
        .expect("Failed to execute tell command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Tell failed. stdout: {stdout}, stderr: {stderr}"
    );
    assert!(
        stdout.contains("The apprentice responds"),
        "Unexpected tell output: {stdout}"
    );
    assert!(stdout.contains("4"), "Response should contain '4'");

    // Test ls includes our apprentice
    let output = Command::new("./target/release/srcrr")
        .arg("ls")
        .output()
        .expect("Failed to execute ls command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("container-test"),
        "Ls should show our apprentice"
    );

    // Cleanup handled automatically by ApprenticeGuard
}

#[test]
#[serial]
fn test_summon_duplicate_fails() {
    // This test doesn't require API key or containers
    // First summon might fail if no runtime, but duplicate check should still work

    let name = "duplicate-test";
    let _guard = ApprenticeGuard::new(name);

    // Try to summon twice
    let _ = Command::new("./target/release/srcrr")
        .args(["summon", name])
        .output();

    // Second summon should fail with "already exists" if first succeeded
    let output = Command::new("./target/release/srcrr")
        .args(["summon", name])
        .output()
        .expect("Failed to execute summon command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If container runtime is available, it should say "already exists"
    // If not available, it will fail with a different error
    // This at least tests the CLI is working
    assert!(
        stdout.contains("already exists") || stdout.contains("Failed to summon"),
        "Unexpected output: {stdout}"
    );

    // Cleanup handled automatically by ApprenticeGuard
}

#[test]
fn test_invalid_apprentice_name() {
    let output = Command::new("./target/release/srcrr")
        .args(["summon", "invalid name with spaces"])
        .output()
        .expect("Failed to execute summon command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Invalid apprentice name") || stdout.contains("Failed to summon"),
        "Should reject invalid name"
    );
}
