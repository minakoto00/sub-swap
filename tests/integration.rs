#[test]
fn test_full_lifecycle_help() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_sub-swap"))
        .args(["--help"])
        .output()
        .expect("Failed to run sub-swap");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sub-swap"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("use"));
    assert!(stdout.contains("add"));
}

#[test]
fn test_binary_version() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_sub-swap"))
        .args(["--version"])
        .output()
        .expect("Failed to run sub-swap");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sub-swap"));
}
