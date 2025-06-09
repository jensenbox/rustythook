use std::process::Command;
use std::path::Path;

#[test]
fn test_version_output() {
    // Get the version from Cargo.toml
    let cargo_version = env!("CARGO_PKG_VERSION");

    // First, build the binary to ensure it's up to date
    let build_output = Command::new("cargo")
        .args(["build", "--bin", "rh"])
        .output()
        .expect("Failed to build binary");

    assert!(
        build_output.status.success(),
        "Failed to build binary: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    // Get the path to the built binary
    let binary_path = Path::new("target/debug/rh");
    assert!(
        binary_path.exists(),
        "Binary not found at expected path: {:?}",
        binary_path
    );

    // Run the binary with --version flag
    let output = Command::new(binary_path)
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    // Convert the output to a string
    let version_output = String::from_utf8_lossy(&output.stdout);

    // Check if the output contains the version from Cargo.toml
    assert!(
        version_output.contains(cargo_version),
        "Version output '{}' does not contain the expected version '{}'",
        version_output,
        cargo_version
    );

    // Also check that the output format is as expected
    assert!(
        version_output.contains("rustyhook"),
        "Version output '{}' does not contain the application name",
        version_output
    );
}
