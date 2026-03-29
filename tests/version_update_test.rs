// Tests verifying the version bump from 1.5.0 to 1.6.0.
//
// Covers:
//   - The compiled binary reports version 1.6.0 via --version
//   - The CARGO_PKG_VERSION constant embedded at compile time is 1.6.0
//   - README.md Changelog contains a v1.6.0 entry
//   - The v1.6.0 Changelog entry documents the --local / -l option

const BIN: &str = env!("CARGO_BIN_EXE_git-wire");
const EXPECTED_VERSION: &str = "1.6.0";

// --- --version flag ---

#[test]
fn version_flag_outputs_1_6_0() {
    // Given: the binary is invoked with --version
    let output = std::process::Command::new(BIN)
        .arg("--version")
        .output()
        .expect("failed to run binary");

    // Then: exits 0 and stdout contains the expected version string
    assert!(output.status.success(), "--version should exit 0");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(EXPECTED_VERSION),
        "--version output should contain '{EXPECTED_VERSION}', got: {stdout}"
    );
}

// --- CARGO_PKG_VERSION ---

#[test]
fn cargo_pkg_version_is_1_6_0() {
    // Given: the package version embedded at compile time
    let pkg_version = env!("CARGO_PKG_VERSION");

    // Then: it must match the target version
    assert_eq!(
        pkg_version, EXPECTED_VERSION,
        "CARGO_PKG_VERSION should be '{EXPECTED_VERSION}', got '{pkg_version}'"
    );
}

// --- README.md Changelog ---

#[test]
fn readme_changelog_has_v1_6_0_entry() {
    // Given: README.md at the repository root
    let readme = std::fs::read_to_string("README.md").expect("README.md must exist");

    // Then: the Changelog section contains a v1.6.0 entry
    assert!(
        readme.contains("v1.6.0"),
        "README.md Changelog should contain 'v1.6.0'"
    );
}

#[test]
fn readme_changelog_v1_6_0_entry_appears_before_v1_5_0() {
    // Given: README.md at the repository root
    let readme = std::fs::read_to_string("README.md").expect("README.md must exist");

    // Then: v1.6.0 appears before v1.5.0 in the file (newer entries first)
    let pos_1_6_0 = readme.find("v1.6.0").expect("v1.6.0 must exist in README.md");
    let pos_1_5_0 = readme.find("v1.5.0").expect("v1.5.0 must exist in README.md");
    assert!(
        pos_1_6_0 < pos_1_5_0,
        "v1.6.0 entry should appear before v1.5.0 in README.md Changelog"
    );
}

#[test]
fn readme_changelog_v1_6_0_mentions_local_option() {
    // Given: README.md at the repository root
    let readme = std::fs::read_to_string("README.md").expect("README.md must exist");

    // Then: the v1.6.0 section documents the --local option
    // Find the v1.6.0 entry and verify --local appears before the next version entry
    let pos_1_6_0 = readme.find("v1.6.0").expect("v1.6.0 must exist in README.md");
    let pos_1_5_0 = readme.find("v1.5.0").expect("v1.5.0 must exist in README.md");
    let v1_6_0_section = &readme[pos_1_6_0..pos_1_5_0];

    assert!(
        v1_6_0_section.contains("--local"),
        "v1.6.0 Changelog entry should document the --local option\n\
         v1.6.0 section:\n{v1_6_0_section}"
    );
}

#[test]
fn readme_changelog_v1_6_0_mentions_local_short_flag() {
    // Given: README.md at the repository root
    let readme = std::fs::read_to_string("README.md").expect("README.md must exist");

    // Then: the v1.6.0 section also documents the -l short flag
    let pos_1_6_0 = readme.find("v1.6.0").expect("v1.6.0 must exist in README.md");
    let pos_1_5_0 = readme.find("v1.5.0").expect("v1.5.0 must exist in README.md");
    let v1_6_0_section = &readme[pos_1_6_0..pos_1_5_0];

    assert!(
        v1_6_0_section.contains("-l"),
        "v1.6.0 Changelog entry should document the -l short flag\n\
         v1.6.0 section:\n{v1_6_0_section}"
    );
}
