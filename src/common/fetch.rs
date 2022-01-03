use std::process::Command;

use cause::Cause;
use cause::cause;
use temp_dir::TempDir;
use regex::Regex;


use super::ErrorType;
use super::ErrorType::*;
use super::Parsed;

pub fn fetch_target_to_tempdir(parsed: &Parsed)
    -> Result<TempDir, Cause<ErrorType>>
{
    let tempdir = TempDir::new()
        .or_else(|e| Err(cause!(TempDirCreationError).src(e)))?;

    std::env::set_current_dir(tempdir.path())
        .or_else(|e| Err(cause!(GitCheckoutChangeDirectoryError).src(e)))?;

    git_clone(parsed)?;
    git_checkout(parsed)?;

    Ok(tempdir)
}

fn git_clone(parsed: &Parsed) -> Result<(), Cause<ErrorType>> {
    println!("  - clone --no-checkout: {}", parsed.url);

    let out = Command::new("git")
        .args([
            "clone",
            "--depth", "1",
            "--filter=blob:none",
            "--no-checkout",
            parsed.url.as_ref(),
            "./"
        ])
        .output()
        .or_else(|e| Err(cause!(GitCloneCommandError).src(e)))?;

    if out.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8(out.stderr)
            .unwrap_or("Could not get even a error output of git clone command".into());
        Err(cause!(GitCloneCommandExitStatusError, error))
    }
}

fn git_checkout(parsed: &Parsed) -> Result<(), Cause<ErrorType>> {
    let rev = identify_commit_hash(parsed)?;
    let rev = if let Some(r) = rev {
        println!("  - checkout: {} ({})", r, parsed.rev);
        r
    } else {
        println!("  - checkout: {}", parsed.rev);
        parsed.rev.to_owned()
    };

    let out = Command::new("git")
        .args([
            "checkout",
            rev.as_ref(),
            "--",
            parsed.src.as_ref(),
        ])
        .output()
        .or_else(|e| Err(cause!(GitCheckoutCommandError).src(e)))?;

    if out.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8(out.stderr)
            .unwrap_or("Could not get even a error output of git checkout command".into());
        Err(cause!(GitCheckoutCommandExitStatusError, error))
    }
}

fn identify_commit_hash(parsed: &Parsed) -> Result<Option<String>, Cause<ErrorType>> {
    let out = Command::new("git")
        .args([
            "ls-remote",
            "--heads",
            "--tags",
            parsed.url.as_ref()
        ])
        .output()
        .or_else(|e| Err(cause!(GitLsRemoteCommandError).src(e)))?;

    if !out.status.success() {
        let error = String::from_utf8(out.stderr)
            .unwrap_or("Could not get even a error output of git checkout command".into());
        return Err(cause!(GitLsRemoteCommandExitStatusError).msg(error));
    }

    let stdout = String::from_utf8(out.stdout)
        .or_else(|e| Err(cause!(GitLsRemoteCommandStdoutDecodeError).src(e)))?;
    let lines = stdout.lines();

    let re_in_line = Regex::new(&format!("^((?:[0-9a-fA-F]){{40}})\\s+(.*{})(\\^\\{{\\}})?$", parsed.rev))
        .or_else(|e| Err(cause!(GitLsRemoteCommandStdoutRegexError).src(e)))?;

    let matched = lines.filter_map(|l| {
        let cap = re_in_line.captures(l)?;
        let hash = cap.get(1)?.as_str().to_owned();
        let name = cap.get(2)?.as_str().to_owned();

        // Check whether the name is same as `parsed.rev` without doubt,
        // since current regex match method might have some ambiguity.
        // (e.g. if `.` included in 'parsed.rev')
        if !name.contains(&parsed.rev) {
            return None;
        }

        // - A shorter name is supposed that it's more likey exactry matched.
        // - Havinig '^{}' at the end should be selected if the name is same as another.
        let wrongness = (name.len() * 100) + if cap.get(3).is_some() { 0 } else { 1 };

        Some((hash, name, wrongness))
    });
    let identified = matched.min_by(|l, r| l.2.cmp(&r.2));

    if let Some((rev, _, _)) = identified {
        Ok(Some(rev))
    } else {
        // There is no items among refs/heads and refs/tags.
        // `parsed.rev` must be a commit hash value or at least part of that.
        Ok(None)
    }
}
