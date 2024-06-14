use std::sync::Arc;

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

pub trait Operation {
    fn operate(
        &self,
        prefix: &str,
        parsed: &Parsed,
        rootdir: &String,
        tempdir: &TempDir,
    ) -> Result<bool, Cause<ErrorType>>;
}

pub fn sequence(
    name: Option<String>,
    operation: Arc<dyn Operation + Send + Sync>,
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
        Mode::Serialized => serialized(parsed, rootdir, operation),
        Mode::Parallel => parallel(parsed, rootdir, operation),
    }
}

fn serialized(
    parsed: Vec<Parsed>,
    rootdir: String,
    operation: Arc<dyn Operation>,
) -> Result<bool, Cause<ErrorType>> {
    let len = parsed.len();

    let mut result = true;
    for (i, parsed) in parsed.iter().enumerate() {
        println!(">> {}/{} started{}", i + 1, len, additional_message_at_start(&parsed));
        let tempdir = common::fetch::fetch_target_to_tempdir("", parsed)?;
        let success = operation.operate("", &parsed, &rootdir, &tempdir)?;
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
    operation: Arc<dyn Operation + Send + Sync>,
) -> Result<bool, Cause<ErrorType>> {
    let len = parsed.len();
    let operation = operation.clone();

    let mut result = true;
    std::thread::scope(|s| {
        let r: Vec<_> = parsed.into_iter().enumerate().map(|(i, parsed)| {
            s.spawn({
                let rootdir = rootdir.clone();
                let operation = operation.clone();
                move || -> Result<bool, Cause<ErrorType>> {
                    let prefix = format!("No.{i} ");
                    println!(">> {}({}/{}) started{}", prefix, i + 1, len, additional_message_at_start(&parsed));
                    let tempdir = common::fetch::fetch_target_to_tempdir(&prefix, &parsed)?;
                    let success = operation.operate(&prefix, &parsed, &rootdir, &tempdir)?;
                    if !success {
                        result = false;
                    }
                    Ok(result)
                }
            })
        }).collect();
        let r: Vec<_> = r.into_iter().map(|h| h.join()).collect();
        println!("{r:?}");
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
