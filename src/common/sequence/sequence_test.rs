// Tests for Target::Local routing inside sequence().
//
// Network-dependent paths (git fetch) cannot be tested here, so coverage is
// limited to:
//   1. Error propagation when .gitwire_local is absent
//   2. Name-filter logic: NoItemToOperateError when no item matches the name
//
// FILE_LOCK is shared with parse_test to prevent concurrent cwd file access.
use std::sync::Arc;

use cause::Cause;
use temp_dir::TempDir;

use crate::common::{ErrorType, Parsed, Target};
use crate::common::ErrorType::*;
use crate::common::parse::parse_test::FILE_LOCK;
use super::{sequence, Mode, Operation};

const NAMED_FOO_ITEM_JSON: &str =
    r#"[{"name":"foo","url":"https://example.com/repo","rev":"main","src":"src","dst":"dst"}]"#;

struct NeverCalledOperation;

impl Operation for NeverCalledOperation {
    fn operate(
        &self,
        _prefix: &str,
        _parsed: &Parsed,
        _rootdir: &String,
        _tempdir: &TempDir,
    ) -> Result<bool, Cause<ErrorType>> {
        panic!("Operation.operate must not be called in this test");
    }
}

fn gitwire_local_path() -> std::path::PathBuf {
    std::env::current_dir().unwrap().join(".gitwire_local")
}

// --- Target::Local(None): error propagation from parse_gitwire_local ---

#[test]
fn sequence_target_local_none_returns_parse_error_when_file_absent() {
    // Given: .gitwire_local does not exist
    let _lock = FILE_LOCK.lock().unwrap();
    let path = gitwire_local_path();
    let _ = std::fs::remove_file(&path);

    // When: Target::Local(None) is passed to sequence
    let op = Arc::new(NeverCalledOperation);
    let result = sequence(Target::Local(None), op, Mode::Single);

    // Then: DotGitWireFileOpenError bubbles up (not NoItemToOperateError or other)
    assert!(matches!(*result.unwrap_err(), DotGitWireFileOpenError));
}

// --- Target::Local(Some(name)): error propagation from parse_gitwire_local ---

#[test]
fn sequence_target_local_some_name_returns_parse_error_when_file_absent() {
    // Given: .gitwire_local does not exist
    let _lock = FILE_LOCK.lock().unwrap();
    let path = gitwire_local_path();
    let _ = std::fs::remove_file(&path);

    // When: Target::Local(Some("foo")) is passed to sequence
    let op = Arc::new(NeverCalledOperation);
    let result = sequence(Target::Local(Some("foo".to_string())), op, Mode::Single);

    // Then: DotGitWireFileOpenError bubbles up
    assert!(matches!(*result.unwrap_err(), DotGitWireFileOpenError));
}

// --- Target::Local(Some(name)): name filter removes all items ---

#[test]
fn sequence_target_local_some_name_returns_no_item_error_when_name_not_found_in_file() {
    // Given: .gitwire_local exists with an item named "foo"
    let _lock = FILE_LOCK.lock().unwrap();
    let path = gitwire_local_path();
    std::fs::write(&path, NAMED_FOO_ITEM_JSON).unwrap();

    // When: sequence is called with a name that does not match any item
    let op = Arc::new(NeverCalledOperation);
    let result = sequence(Target::Local(Some("bar".to_string())), op, Mode::Single);
    std::fs::remove_file(&path).ok();

    // Then: the filter produces an empty list → NoItemToOperateError
    assert!(matches!(*result.unwrap_err(), NoItemToOperateError));
}

// --- Target::Direct(parsed): routing bypasses .gitwire file parsing ---

#[test]
fn sequence_target_direct_bypasses_file_parsing_and_always_has_one_item() {
    // Given: a Parsed value supplied directly; no .gitwire / .gitwire_local is required,
    // and Direct always wraps exactly one item so NoItemToOperateError cannot occur.
    let _lock = FILE_LOCK.lock().unwrap();
    let saved_cwd = std::env::current_dir().unwrap();

    let parsed = Parsed {
        name: None,
        dsc: None,
        mtd: None,
        url: "not-a-url".to_string(),
        rev: "main".to_string(),
        src: "src".to_string(),
        dst: "dst".to_string(),
    };
    let op = Arc::new(NeverCalledOperation);

    // When: sequence is called with Target::Direct (neither .gitwire file is read)
    let result = sequence(Target::Direct(parsed), op, Mode::Single);

    // fetch_target_to_tempdir changes the process cwd; restore it for subsequent tests
    let _ = std::env::set_current_dir(&saved_cwd);

    // Then: error originates from the git network operation, not from file parsing or
    // an empty item list.
    let err = result.unwrap_err();
    assert!(
        !matches!(*err, DotGitWireFileOpenError),
        "Target::Direct must not trigger DotGitWireFileOpenError"
    );
    assert!(
        !matches!(*err, NoItemToOperateError),
        "Target::Direct must never produce NoItemToOperateError"
    );
}
