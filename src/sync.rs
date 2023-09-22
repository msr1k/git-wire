use std::path::Path;
use std::path::PathBuf;

use crate::common;
use cause::Cause;
use cause::cause;
use fs_extra;

use crate::common::Parsed;
use crate::common::ErrorType;
use crate::common::ErrorType::*;

pub fn sync() -> Result<bool, Cause<ErrorType>> {
    println!("git-wire sync started\n");
    let (rootdir, parsed) = common::parse::parse_gitwire()?;

    let len = parsed.len();
    for (i, parsed) in parsed.iter().enumerate() {
        println!(">> {}/{} started", i + 1, len);
        let tempdir = common::fetch::fetch_target_to_tempdir(&parsed)?;
        move_from_temp(&parsed, &rootdir, tempdir.path())?;
    }
    println!(">> All sync tasks have done!\n");
    Ok(true)
}

fn move_from_temp(parsed: &Parsed, root: &str, temp: &Path) -> Result<(), Cause<ErrorType>> {
    println!("  - copy from `src` to `dst`");

    let from = temp.join(parsed.src.as_str());
    let to = PathBuf::from(root).join(parsed.dst.as_str());

    let mut opt = fs_extra::dir::CopyOptions::new();
    opt.overwrite = true;
    opt.copy_inside = true;

    fs_extra::remove_items(&[&to])
        .or_else(|e| {
            let cause = cause!(MoveFromTempToDestError).src(e)
                .msg(format!("Could not remove {:?}", to));
            Err(cause)
        })?;

    fs_extra::move_items(&[&from], &to, &opt)
        .or_else(|e| {
            let cause = cause!(MoveFromTempToDestError).src(e)
                .msg(format!("Could not copy from {:?} to {:?}", from, to));
            Err(cause)
        })?;
    Ok(())
}
