use std::sync::Arc;

use cause::Cause;
use cause::cause;
use temp_dir::TempDir;

use crate::common;
use crate::common::Target;
use crate::common::Parsed;
use crate::common::ErrorType;
use crate::common::ErrorType::*;

pub enum Mode {
    Single,
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
    target: Target,
    operation: Arc<dyn Operation + Send + Sync>,
    mode: Mode, 
) -> Result<bool, Cause<ErrorType>> {

    let (rootdir, parsed): (String, Vec<_>) = match target {
        Target::Declared(Some(ref name)) => {
            let (rootdir, parsed) = common::parse::parse_gitwire()?;
            let parsed = parsed.into_iter()
                .filter(|p| {
                    match p.name {
                        Some(ref n) => n == name,
                        None => false,
                    }
                })
                .collect();
            (rootdir, parsed)
        },
        Target::Declared(None) => {
            common::parse::parse_gitwire()?
        },
        Target::Direct(parsed) => {
            (std::env::current_dir().unwrap().into_os_string().into_string().unwrap(), vec![parsed]) // TODO
        },
    };

    let len = parsed.len();
    if len == 0 {
        Err(cause!(NoItemToOperateError, "There are no items to operate."))?
    }

    match mode {
        Mode::Single => single(parsed, rootdir, operation),
        Mode::Parallel => parallel(parsed, rootdir, operation),
    }
}

fn single(
    parsed: Vec<Parsed>,
    rootdir: String,
    operation: Arc<dyn Operation>,
) -> Result<bool, Cause<ErrorType>> {
    let len = parsed.len();

    let mut result = true;
    for (i, parsed) in parsed.iter().enumerate() {
        println!(">> {}/{} started{}", i + 1, len, additional_message(&parsed));
        let tempdir = common::fetch::fetch_target_to_tempdir("", parsed)?;
        let success = operation.operate("", &parsed, &rootdir, &tempdir)?;
        if !success {
            result = false;
        }
        println!("");
    }
    println!(">> All check tasks have done!\n");
    Ok(result)
}

fn parallel(
    parsed: Vec<Parsed>,
    rootdir: String,
    operation: Arc<dyn Operation + Send + Sync>,
) -> Result<bool, Cause<ErrorType>> {
    use colored::*;

    let len = parsed.len();
    let operation = operation.clone();

    let results: Vec<_> = std::thread::scope(|s| {
        let results: Vec<_> = parsed.into_iter().enumerate().map(|(i, parsed)| {
            s.spawn({
                let rootdir = rootdir.clone();
                let operation = operation.clone();
                move || -> Result<bool, Cause<ErrorType>> {
                    let prefix = format!("No.{i} ");
                    println!("{}", format!(">> {}({}/{}) started{}", prefix, i + 1, len, additional_message(&parsed)).blue());
                    let tempdir = common::fetch::fetch_target_to_tempdir(&prefix, &parsed)?;
                    let success = operation.operate(&prefix, &parsed, &rootdir, &tempdir)?;
                    if success {
                        println!("{}", format!(">> {}({}/{}) succeeded{}", prefix, i + 1, len, additional_message(&parsed)).blue());
                        Ok(true)
                    } else {
                        println!("{}", format!(">> {}({}/{}) failed{}", prefix, i + 1, len, additional_message(&parsed)).magenta());
                        Ok(false)
                    }
                }
            })
        }).collect();
        results.into_iter().map(|h| h.join().unwrap()).collect()
    });
    println!("{}", format!(">> All check tasks have done!\n").blue());

    let result = if let Some(_) = results.iter().find(|r| matches!(r, Ok(false))) {
        Ok(false)
    } else {
        Ok(true)
    };
    if let Some(err) = results.into_iter().find(|r| matches!(r, Err(..))) {
        return err;
    }
    result
}

fn additional_message(parsed: &Parsed) -> String {
    match (&parsed.name, &parsed.dsc) {
        (Some(name), Some(dsc))     => format!(" ({}: {})", name, dsc),
        (Some(name), None)          => format!(" ({})",     name),
        (None,       Some(ref dsc)) => format!(" ({})",     dsc),
        (None,       None)          => "".to_owned(),
    }
}
