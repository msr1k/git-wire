// Tests for parse_gitwire_local().
//
// File-based tests serialize via FILE_LOCK because all unit tests in the
// binary share the same cwd, and parse_gitwire_local() reads .gitwire_local
// from cwd.  Any test that creates the file acquires the lock first.
use std::sync::Mutex;

use super::*;
use crate::common::ErrorType::*;

pub(crate) static FILE_LOCK: Mutex<()> = Mutex::new(());

const VALID_ITEM_JSON: &str =
    r#"[{"url":"https://example.com/repo","rev":"main","src":"src","dst":"dst"}]"#;
const INVALID_JSON: &str = r#"not valid json"#;
const DOTGIT_SRC_JSON: &str =
    r#"[{"url":"https://example.com","rev":"main","src":".git/hooks","dst":"dst"}]"#;
const PARENT_DIR_SRC_JSON: &str =
    r#"[{"url":"https://example.com","rev":"main","src":"../outside","dst":"dst"}]"#;
const DUPLICATE_NAME_JSON: &str = concat!(
    r#"[{"name":"dup","url":"https://example.com","rev":"main","src":"a","dst":"b"},"#,
    r#"{"name":"dup","url":"https://example.com","rev":"main","src":"c","dst":"d"}]"#,
);

fn gitwire_local_path() -> std::path::PathBuf {
    std::env::current_dir().unwrap().join(".gitwire_local")
}

// --- error: file not found ---

#[test]
fn parse_gitwire_local_returns_error_when_file_not_found() {
    // Given: .gitwire_local does not exist in cwd
    let _lock = FILE_LOCK.lock().unwrap();
    let path = gitwire_local_path();
    let _ = std::fs::remove_file(&path);

    // When
    let result = parse_gitwire_local();

    // Then
    assert!(matches!(*result.unwrap_err(), DotGitWireFileOpenError));
}

// --- success: valid file ---

#[test]
fn parse_gitwire_local_returns_parsed_items_when_valid_file_exists() {
    // Given: valid .gitwire_local in cwd
    let _lock = FILE_LOCK.lock().unwrap();
    let path = gitwire_local_path();
    std::fs::write(&path, VALID_ITEM_JSON).unwrap();

    // When
    let result = parse_gitwire_local();
    std::fs::remove_file(&path).ok();

    // Then: parsed items reflect file content
    let (_, items) = result.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].url, "https://example.com/repo");
    assert_eq!(items[0].rev, "main");
    assert_eq!(items[0].src, "src");
    assert_eq!(items[0].dst, "dst");
}

#[test]
fn parse_gitwire_local_root_is_current_dir_not_git_toplevel() {
    // Given: valid .gitwire_local in cwd
    let _lock = FILE_LOCK.lock().unwrap();
    let cwd = std::env::current_dir().unwrap();
    let path = cwd.join(".gitwire_local");
    std::fs::write(&path, VALID_ITEM_JSON).unwrap();

    // When
    let result = parse_gitwire_local();
    std::fs::remove_file(&path).ok();

    // Then: root equals current_dir() (not the result of git rev-parse --show-toplevel)
    let (root, _) = result.unwrap();
    let expected = cwd.into_os_string().into_string().unwrap();
    assert_eq!(root, expected);
}

// --- error: invalid JSON ---

#[test]
fn parse_gitwire_local_returns_error_for_invalid_json() {
    // Given: .gitwire_local contains invalid JSON
    let _lock = FILE_LOCK.lock().unwrap();
    let path = gitwire_local_path();
    std::fs::write(&path, INVALID_JSON).unwrap();

    // When
    let result = parse_gitwire_local();
    std::fs::remove_file(&path).ok();

    // Then
    assert!(matches!(*result.unwrap_err(), DotGitWireFileParseError));
}

// --- error: soundness check ---

#[test]
fn parse_gitwire_local_returns_soundness_error_when_src_contains_dotgit() {
    // Given: src path contains .git component
    let _lock = FILE_LOCK.lock().unwrap();
    let path = gitwire_local_path();
    std::fs::write(&path, DOTGIT_SRC_JSON).unwrap();

    // When
    let result = parse_gitwire_local();
    std::fs::remove_file(&path).ok();

    // Then
    assert!(matches!(*result.unwrap_err(), DotGitWireFileSoundnessError));
}

#[test]
fn parse_gitwire_local_returns_soundness_error_when_src_contains_parent_dir() {
    // Given: src path contains .. component
    let _lock = FILE_LOCK.lock().unwrap();
    let path = gitwire_local_path();
    std::fs::write(&path, PARENT_DIR_SRC_JSON).unwrap();

    // When
    let result = parse_gitwire_local();
    std::fs::remove_file(&path).ok();

    // Then
    assert!(matches!(*result.unwrap_err(), DotGitWireFileSoundnessError));
}

// --- error: duplicate names ---

#[test]
fn parse_gitwire_local_returns_name_not_unique_error_for_duplicate_names() {
    // Given: two items share the same name
    let _lock = FILE_LOCK.lock().unwrap();
    let path = gitwire_local_path();
    std::fs::write(&path, DUPLICATE_NAME_JSON).unwrap();

    // When
    let result = parse_gitwire_local();
    std::fs::remove_file(&path).ok();

    // Then
    assert!(matches!(*result.unwrap_err(), DotGitWireFileNameNotUniqueError));
}
