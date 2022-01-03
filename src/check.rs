use std::path::Path;

use cause::Cause;
use cause::cause;
use folder_compare::FolderCompare;
use regex::Regex;

use crate::common;
use crate::common::Parsed;
use crate::common::ErrorType;
use crate::common::ErrorType::*;

pub fn check() -> Result<bool, Cause<ErrorType>> {
    println!("git-wire check startd\n");
    let (rootdir, parsed) = common::parse::parse_gitwire()?;

    let len = parsed.len();
    let mut result = true;
    for (i, parsed) in parsed.iter().enumerate() {
        println!(">> {}/{} started", i + 1, len);
        let tempdir = common::fetch::fetch_target_to_tempdir(&parsed)?;
        let no_diff = compare_with_temp(&parsed, &rootdir, tempdir.path())?;
        if result && !no_diff {
            result = false;
        }
    }
    println!(">> All check tasks have done!\n");
    Ok(result)
}

fn compare_with_temp(parsed: &Parsed, root: &str, temp: &Path) -> Result<bool, Cause<ErrorType>> {
    println!("  - compare `src` and `dst`");

    let temp = temp.join(parsed.src.clone());
    let root = Path::new(root).join(parsed.dst.clone());

    let fc1 = FolderCompare::new(&temp, &root, &vec![".git".into()])
        .or_else(|e| Err(cause!(CheckDifferenceExecutionError(e))))?;
    let fc2 = FolderCompare::new(&root, &temp, &vec![".git".into()])
        .or_else(|e| Err(cause!(CheckDifferenceExecutionError(e))))?;

    let mut result = true;

    if fc1.new_files.len() > 0 {
        for file in fc1.new_files {
            let file = file.to_str()
                .ok_or_else(|| cause!(CheckDifferenceStringReplaceError))?;
            let re = Regex::new(r"^.*?git-wire[^/]*/")
                .or_else(|e| Err(cause!(CheckDifferenceStringReplaceError).src(e)))?;
            let file = re.replace(file, "");
            println!("  ! file {:?} does not exist", file);
        }
        result = false;
    }
    if fc2.new_files.len() > 0 {
        for file in fc2.new_files {
            println!("  ! file {:?} does not exist on original", file);
        }
        result = false;
    }
    if fc2.changed_files.len() > 0 {
        for file in fc2.changed_files {
            println!("  ! file {:?} is not identical to original", file);
        }
        result = false;
    }

    Ok(result)
}
