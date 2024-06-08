use std::process::Command;
use std::fs::File;
use std::path::Path;
use std::io::BufReader;

use cause::Cause;
use cause::cause;

use super::ErrorType::{self, *};
use super::Parsed;

const DOT_GIT_WIRE: &str = ".gitwire";

pub fn parse_gitwire() -> Result<(String, Vec<Parsed>), Cause<ErrorType>> {
    let (root, file) = get_dotgitwire_file_path()?;
    Ok((root, parse_dotgitwire_file(file)?))
}

fn get_dotgitwire_file_path() -> Result<(String, String), Cause<ErrorType>> {
    let out = Command::new("git")
        .args([ "rev-parse", "--show-toplevel" ])
        .output()
        .or_else(|e| Err(cause!(RepositoryRootPathCommandError).src(e)))?
        .stdout;

    let root = remove_line_ending(
        String::from_utf8(out)
        .or_else(|e| Err(cause!(RepositoryRootPathParseError).src(e)))?
    );
    let root = String::from(root);

    let file = format!("{}/{}", root, DOT_GIT_WIRE);
    if !Path::new(&file).exists() {
        return Err(cause!(DotGitWireFileOpenError, "There is no .gitwire file in this repository"));
    }
    Ok((root, file))
}

fn parse_dotgitwire_file(file: String) -> Result<Vec<Parsed>, Cause<ErrorType>> {
    let f = File::open(file)
        .or_else(|e| Err(cause!(DotGitWireFileOpenError, "no .gitwire file read permission").src(e)))?;
    let reader = BufReader::new(f);
    let parsed: Vec<Parsed> = serde_json::from_reader(reader)
        .or_else(|e| Err(cause!(DotGitWireFileParseError, ".gitwire file format is wrong").src(e)))?;

    if parsed.iter().any(|item| !check_parsed_item_soundness(item)) {
        Err(cause!(DotGitWireFileSoundnessError, ".gitwire file's `src` and `dst` must not include '.', '..', and '.git'."))?
    }

    let deduplicated_count = parsed.iter()
        .filter_map(|p| p.name.as_ref().map(|name| name.as_str()))
        .collect::<std::collections::HashSet<&str>>().iter()
        .count();
    if parsed.iter().filter(|p| p.name.is_some()).count() != deduplicated_count {
        Err(cause!(DotGitWireFileNameNotUniqueError, ".gitwire file's `name`s must be differ each other."))?
    }

    Ok(parsed)
}

fn remove_line_ending(string: String) -> String {
    string
        .strip_suffix("\r\n")
        .or(string.strip_suffix("\n"))
        .unwrap_or(string.as_ref())
        .into()
}

use std::path::Component;
use std::ffi::OsStr;

fn check_parsed_item_soundness(parsed: &Parsed) -> bool {
    let is_ok = |e: &Component| -> bool {
        match e {
            Component::Prefix(_) => true,
            Component::RootDir => true,
            Component::Normal(name) => name.ne(&OsStr::new(".git")),
            Component::ParentDir => false,
            Component::CurDir => false
        }
    };
    let src_result_ok = Path::new(&parsed.src).components().all(|p| is_ok(&p));
    let dst_result_ok = Path::new(&parsed.dst).components().all(|p| is_ok(&p));
    src_result_ok && dst_result_ok
}
