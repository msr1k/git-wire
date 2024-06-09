use cause::Cause;
use cause::cause;
use temp_dir::TempDir;

use crate::common;
use crate::common::Parsed;
use crate::common::ErrorType;
use crate::common::ErrorType::*;

pub fn sequence(
    name: Option<String>,
    func: impl Fn(&Parsed, &String, &TempDir) -> Result<bool, Cause<ErrorType>>,
) -> Result<bool, Cause<ErrorType>> {
    let (rootdir, parsed) = common::parse::parse_gitwire()?;

    let parsed: Vec<_> = if name.is_some() {
        parsed.into_iter()
            .filter(|p| p.name.is_some() && p.name == name)
            .collect()
    } else {
        parsed
    };

    let len = parsed.len();
    if len == 0 {
        Err(cause!(NoItemToOperateError, "There are no items to operate."))?
    }

    let mut result = true;
    for (i, parsed) in parsed.iter().enumerate() {
        let name_str = match (&parsed.name, &parsed.dsc) {
            (Some(name), Some(dsc))     => format!(" ({}: {})", name, dsc),
            (Some(name), None)          => format!(" ({})",     name),
            (None,       Some(ref dsc)) => format!(" ({})",     dsc),
            (None,       None)          => "".to_owned(),
        };
        println!(">> {}/{} started{}", i + 1, len, name_str);
        let tempdir = common::fetch::fetch_target_to_tempdir(parsed)?;
        let success = func(&parsed, &rootdir, &tempdir)?;
        if !success {
            result = false;
        }
        println!("");
    }
    Ok(result)
}
