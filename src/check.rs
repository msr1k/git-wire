use std::path::Path;
use std::sync::Arc;

use cause::Cause;
use cause::cause;
use folder_compare::FolderCompare;
use temp_dir::TempDir;

use crate::common;
use crate::common::Parsed;
use crate::common::ErrorType;
use crate::common::ErrorType::*;
use crate::common::sequence::Operation;

#[derive(Debug)]
struct CheckOperation {}

impl Operation for CheckOperation {
    fn operate(
        &self,
        prefix: &str,
        parsed: &Parsed,
        rootdir: &String,
        tempdir: &TempDir,
    ) -> Result<bool, Cause<ErrorType>> {
        compare_with_temp(
            prefix,
            parsed,
            rootdir,
            tempdir.path(),
        )
    }
}


pub fn check(name: Option<String>, mode: common::sequence::Mode) -> Result<bool, Cause<ErrorType>> {
    println!("git-wire check started\n");
    let operation = Arc::new(CheckOperation {});
    let result = common::sequence::sequence(
        name,
        operation,
        mode,
    )?;
    Ok(result)
}

fn compare_with_temp(prefix: &str, parsed: &Parsed, root: &str, temp: &Path) -> Result<bool, Cause<ErrorType>> {
    println!("  - {prefix}compare `src` and `dst`");

    let temp_root = temp;
    let temp = temp.join(parsed.src.as_str());
    let root = Path::new(root).join(parsed.dst.as_str());

    let fc1 = FolderCompare::new(&temp, &root, &vec![])
        .or_else(|e| Err(cause!(CheckDifferenceExecutionError(e))))?;
    let fc2 = FolderCompare::new(&root, &temp, &vec![])
        .or_else(|e| Err(cause!(CheckDifferenceExecutionError(e))))?;

    let mut result = true;

    use colored::*;

    if fc1.new_files.len() > 0 {
        let temp_root = temp_root.to_str()
            .ok_or_else(|| cause!(CheckDifferenceStringReplaceError))?;
        for file in fc1.new_files {
            let file = file.to_str()
                .ok_or_else(|| cause!(CheckDifferenceStringReplaceError))?;
            let file = file.replace(temp_root, "");
            println!("{}", format!("    {prefix}! file {:?} does not exist", file).red());
        }
        result = false;
    }
    if fc2.new_files.len() > 0 {
        for file in fc2.new_files {
            println!("{}", format!("    {prefix}! file {:?} does not exist on original", file).red());
        }
        result = false;
    }
    if fc2.changed_files.len() > 0 {
        for file in fc2.changed_files {
            println!("{}", format!("    {prefix}! file {:?} is not identical to original", file).red());
        }
        result = false;
    }

    Ok(result)
}
