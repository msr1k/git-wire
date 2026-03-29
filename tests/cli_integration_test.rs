// Integration tests for CLI commands.
//
// These tests run the compiled binary to verify that the subcommands are
// recognised, parse their required arguments correctly, and produce useful
// error messages when arguments are missing.
//

const BIN: &str = env!("CARGO_BIN_EXE_git-wire");

// --- direct-sync: subcommand is recognised ---

#[test]
fn direct_sync_without_args_reports_missing_args_not_unknown_subcommand() {
    // Given: direct-sync called without any required arguments
    let output = std::process::Command::new(BIN)
        .args(["direct-sync"])
        .output()
        .expect("failed to run binary");

    // Then: the binary exits with an error about missing arguments, not
    //       "unrecognized subcommand" (which would mean the command was never added)
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        !stderr.contains("unrecognized subcommand"),
        "unexpected 'unrecognized subcommand' in stderr: {stderr}"
    );
}

// --- direct-sync: --help shows all four required args ---

#[test]
fn direct_sync_help_lists_url_rev_src_dst_args() {
    // Given: direct-sync --help
    let output = std::process::Command::new(BIN)
        .args(["direct-sync", "--help"])
        .output()
        .expect("failed to run binary");

    // Then: exits 0 and stdout documents all required flags
    assert!(output.status.success(), "direct-sync --help should exit 0");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--url"),  "help should mention --url\n{stdout}");
    assert!(stdout.contains("--rev"),  "help should mention --rev\n{stdout}");
    assert!(stdout.contains("--src"),  "help should mention --src\n{stdout}");
    assert!(stdout.contains("--dst"),  "help should mention --dst\n{stdout}");
}

// --- direct-sync: each required arg is individually enforced ---

#[test]
fn direct_sync_fails_when_url_is_missing() {
    // Given: direct-sync without --url
    let output = std::process::Command::new(BIN)
        .args(["direct-sync", "--rev", "main", "--src", "src", "--dst", "dst"])
        .output()
        .expect("failed to run binary");

    // Then: non-zero exit
    assert!(!output.status.success());
}

#[test]
fn direct_sync_fails_when_rev_is_missing() {
    // Given: direct-sync without --rev
    let output = std::process::Command::new(BIN)
        .args(["direct-sync", "--url", "https://example.com/r", "--src", "src", "--dst", "dst"])
        .output()
        .expect("failed to run binary");

    assert!(!output.status.success());
}

#[test]
fn direct_sync_fails_when_src_is_missing() {
    // Given: direct-sync without --src
    let output = std::process::Command::new(BIN)
        .args(["direct-sync", "--url", "https://example.com/r", "--rev", "main", "--dst", "dst"])
        .output()
        .expect("failed to run binary");

    assert!(!output.status.success());
}

#[test]
fn direct_sync_fails_when_dst_is_missing() {
    // Given: direct-sync without --dst
    let output = std::process::Command::new(BIN)
        .args(["direct-sync", "--url", "https://example.com/r", "--rev", "main", "--src", "src"])
        .output()
        .expect("failed to run binary");

    assert!(!output.status.success());
}

// --- direct-check: subcommand is recognised ---

#[test]
fn direct_check_without_args_reports_missing_args_not_unknown_subcommand() {
    // Given: direct-check called without any required arguments
    let output = std::process::Command::new(BIN)
        .args(["direct-check"])
        .output()
        .expect("failed to run binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        !stderr.contains("unrecognized subcommand"),
        "unexpected 'unrecognized subcommand' in stderr: {stderr}"
    );
}

// --- direct-check: --help shows all four required args ---

#[test]
fn direct_check_help_lists_url_rev_src_dst_args() {
    // Given: direct-check --help
    let output = std::process::Command::new(BIN)
        .args(["direct-check", "--help"])
        .output()
        .expect("failed to run binary");

    assert!(output.status.success(), "direct-check --help should exit 0");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--url"),  "help should mention --url\n{stdout}");
    assert!(stdout.contains("--rev"),  "help should mention --rev\n{stdout}");
    assert!(stdout.contains("--src"),  "help should mention --src\n{stdout}");
    assert!(stdout.contains("--dst"),  "help should mention --dst\n{stdout}");
}

// --- direct-sync / direct-check: --local flag is accepted (global flag) ---

#[test]
fn direct_sync_accepts_local_flag_at_parse_level() {
    // Given: --local is a global flag; it must be accepted with direct-sync
    // even though the implementation ignores it for Target::Direct
    let output = std::process::Command::new(BIN)
        .args(["direct-sync", "--local", "--help"])
        .output()
        .expect("failed to run binary");

    // Then: exits 0 (--local does not cause a parse error for this subcommand)
    assert!(
        output.status.success(),
        "--local should not cause a parse error for direct-sync"
    );
}

#[test]
fn direct_check_accepts_local_flag_at_parse_level() {
    // Given: --local is a global flag; it must be accepted with direct-check
    let output = std::process::Command::new(BIN)
        .args(["direct-check", "--local", "--help"])
        .output()
        .expect("failed to run binary");

    assert!(
        output.status.success(),
        "--local should not cause a parse error for direct-check"
    );
}

// --- sync: --local flag is accepted ---

#[test]
fn sync_help_mentions_local_flag() {
    // Given: sync --help
    let output = std::process::Command::new(BIN)
        .args(["sync", "--help"])
        .output()
        .expect("failed to run binary");

    // Then: exits 0 and help text documents the --local / -l flag
    assert!(output.status.success(), "sync --help should exit 0");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--local"), "sync --help should mention --local\n{stdout}");
    assert!(stdout.contains("-l"), "sync --help should mention -l\n{stdout}");
}

#[test]
fn sync_accepts_local_flag_at_parse_level() {
    // Given: --local is a global flag; it must be accepted with sync
    let output = std::process::Command::new(BIN)
        .args(["sync", "--local", "--help"])
        .output()
        .expect("failed to run binary");

    // Then: exits 0 (--local does not cause a parse error)
    assert!(
        output.status.success(),
        "--local should not cause a parse error for sync"
    );
}

#[test]
fn sync_accepts_local_short_flag_at_parse_level() {
    // Given: -l is the short form of --local; it must be accepted with sync
    let output = std::process::Command::new(BIN)
        .args(["sync", "-l", "--help"])
        .output()
        .expect("failed to run binary");

    // Then: exits 0 (-l does not cause a parse error)
    assert!(
        output.status.success(),
        "-l should not cause a parse error for sync"
    );
}

// --- check: --local flag is accepted ---

#[test]
fn check_help_mentions_local_flag() {
    // Given: check --help
    let output = std::process::Command::new(BIN)
        .args(["check", "--help"])
        .output()
        .expect("failed to run binary");

    // Then: exits 0 and help text documents the --local / -l flag
    assert!(output.status.success(), "check --help should exit 0");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--local"), "check --help should mention --local\n{stdout}");
    assert!(stdout.contains("-l"), "check --help should mention -l\n{stdout}");
}

#[test]
fn check_accepts_local_flag_at_parse_level() {
    // Given: --local is a global flag; it must be accepted with check
    let output = std::process::Command::new(BIN)
        .args(["check", "--local", "--help"])
        .output()
        .expect("failed to run binary");

    // Then: exits 0 (--local does not cause a parse error)
    assert!(
        output.status.success(),
        "--local should not cause a parse error for check"
    );
}

#[test]
fn check_accepts_local_short_flag_at_parse_level() {
    // Given: -l is the short form of --local; it must be accepted with check
    let output = std::process::Command::new(BIN)
        .args(["check", "-l", "--help"])
        .output()
        .expect("failed to run binary");

    // Then: exits 0 (-l does not cause a parse error)
    assert!(
        output.status.success(),
        "-l should not cause a parse error for check"
    );
}
