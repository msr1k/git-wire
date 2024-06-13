use cause::Cause;
use cause::cause;
use temp_dir::TempDir;

use crate::common;
use crate::common::Parsed;
use crate::common::ErrorType;
use crate::common::ErrorType::*;

pub enum Mode {
    Serialized,
    Parallel,
}

pub fn sequence(
    name: Option<String>,
    func: impl Fn(&str, &Parsed, &String, &TempDir) -> Result<bool, Cause<ErrorType>> + Sync + Send,
    mode: Mode, 
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

    match mode {
        Mode::Serialized => serialized(parsed, rootdir, func),
        Mode::Parallel => parallel(parsed, rootdir, func),
    }
}

fn serialized(
    parsed: Vec<Parsed>,
    rootdir: String,
    func: impl Fn(&str, &Parsed, &String, &TempDir) -> Result<bool, Cause<ErrorType>>,
) -> Result<bool, Cause<ErrorType>> {
    let len = parsed.len();

    let mut result = true;
    for (i, parsed) in parsed.iter().enumerate() {
        println!(">> {}/{} started{}", i + 1, len, additional_message_at_start(&parsed));
        let tempdir = common::fetch::fetch_target_to_tempdir("", parsed)?;
        let success = func("", &parsed, &rootdir, &tempdir)?;
        if !success {
            result = false;
        }
        println!("");
    }
    Ok(result)
}

fn parallel(
    parsed: Vec<Parsed>,
    rootdir: String,
    func: impl Fn(&str, &Parsed, &String, &TempDir) -> Result<bool, Cause<ErrorType>> + Sync + Send,
) -> Result<bool, Cause<ErrorType>> {
    let len = parsed.len();
    let func = std::sync::Arc::new(func);

    let mut result = true;
    std::thread::scope(|s| {
        parsed.into_iter().enumerate().for_each(|(i, parsed)| {
            s.spawn({
                let rootdir = rootdir.clone();
                let func = func.clone();
                move || -> Result<bool, Cause<ErrorType>> {
                    let prefix = format!("No.{i} ");
                    println!(">> {}({}/{}) started{}", prefix, i + 1, len, additional_message_at_start(&parsed));
                    let tempdir = common::fetch::fetch_target_to_tempdir(&prefix, &parsed)?;
                    let success = func(&prefix, &parsed, &rootdir, &tempdir)?;
                    if !success {
                        result = false;
                    }
                    Ok(result)
                }
            });
        });
    });
    Ok(result)
}

fn additional_message_at_start(parsed: &Parsed) -> String {
    match (&parsed.name, &parsed.dsc) {
        (Some(name), Some(dsc))     => format!(" ({}: {})", name, dsc),
        (Some(name), None)          => format!(" ({})",     name),
        (None,       Some(ref dsc)) => format!(" ({})",     dsc),
        (None,       None)          => "".to_owned(),
    }
}
