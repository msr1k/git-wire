use std::sync::Arc;
use std::path::Path;

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
        workdir: &Path,
    ) -> Result<bool, Cause<ErrorType>>;
}

fn filter_by_name(parsed: Vec<Parsed>, name: &str) -> Vec<Parsed> {
    parsed.into_iter()
        .filter(|p| matches!(&p.name, Some(n) if n == name))
        .collect()
}

pub fn sequence(
    target: Target,
    operation: Arc<dyn Operation + Send + Sync>,
    mode: Mode, 
) -> Result<bool, Cause<ErrorType>> {

    let (rootdir, parsed): (String, Vec<_>) = match target {
        Target::Declared(Some(ref name)) => {
            let (rootdir, parsed) = common::parse::parse_gitwire()?;
            (rootdir, filter_by_name(parsed, name))
        },
        Target::Declared(None) => {
            common::parse::parse_gitwire()?
        },
        Target::Local(Some(ref name)) => {
            let (rootdir, parsed) = common::parse::parse_gitwire_local()?;
            (rootdir, filter_by_name(parsed, name))
        },
        Target::Local(None) => {
            common::parse::parse_gitwire_local()?
        },
        Target::Direct(parsed) => {
            (
                std::env::current_dir()
                    .or(Err(cause!(ErrorType::CurrentDirRetrieveError)))?
                    .into_os_string()
                    .into_string()
                    .or(Err(cause!(ErrorType::CurrentDirConvertError)))?,
                vec![parsed],
            )
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
        let success = operation.operate("", &parsed, &rootdir, tempdir.path())?;
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
    use std::sync::Mutex;
    use std::collections::HashMap;
    use std::sync::mpsc;

    let len = parsed.len();
    let operation = operation.clone();

    // Channel to send (prefix, parsed, Arc<TempDir>) from producers to consumer
    let dedup_map = Arc::new(Mutex::new(HashMap::<String, Arc<Mutex<Option<TempDir>>>>::new()));

    // Spawn producers to prepare tempdirs (deduplicating by url+rev+mtd) and send messages
    let produce_results: Result<Vec<bool>, Cause<ErrorType>> = std::thread::scope(|s| {
        let handles: Vec<_> = parsed.into_iter().enumerate().map(|(i, parsed)| {
            let (tx, rx) = mpsc::channel::<(String, Parsed, Arc<TempDir>)>();

            s.spawn({
                let dedup_map = dedup_map.clone();
                let prefix = format!("No.{} ", i + 1);

                move || -> Result<bool, Cause<ErrorType>> {
                    println!("{}", format!(">> {}({}/{}) started{}", prefix, i + 1, len, additional_message(&parsed)).blue());

                    // key for deduplication (stable string for Method)
                    let key = create_parallel_fetch_key(&parsed);

                    // Try to get existing TempDir mutex or create it
                    let td_arc_opt = {
                        let mut map = dedup_map.lock().map_err(|_| cause!(ErrorType::TempDirCreationError))?;
                        let val = map.get(&key).cloned();
                        match val {
                            Some(v) => {
                                v.clone()
                            },
                            None => {
                                let v = Arc::new(Mutex::new(None));
                                map.insert(key.clone(), v.clone());
                                v.clone()
                            }
                        }
                    };
                    let mut td_arc_opt = td_arc_opt.lock().unwrap();

                    match td_arc_opt.as_ref() {
                        Some(td_arc) => {
                            // tempdir has been set already
                            // no need to perform fetch, just use it.
                            println!("  - {prefix}reuse existing clone: {} ({})", parsed.url, parsed.rev);
                            tx.send((prefix, parsed, Arc::new(td_arc.clone())))
                                .map_err(|_| cause!(ErrorType::TempDirCreationError))?;
                            Ok(true)
                        },
                        None => {
                            // tempdir has not been set
                            // perform fetch and set earned tempdir value to the map
                            let tempdir = common::fetch::fetch_target_to_tempdir(&prefix, &parsed)?;
                            *td_arc_opt = Some(tempdir.clone());
                            tx.send((prefix, parsed, Arc::new(tempdir)))
                                .map_err(|_| cause!(ErrorType::TempDirCreationError))?;
                            Ok(true)
                        }
                    }
                }
            });

            s.spawn({
                let rootdir = rootdir.clone();
                let operation = operation.clone();

                move || -> Result<bool, Cause<ErrorType>> {
                    if let Ok((prefix, parsed, tempdir_arc)) = rx.recv() {
                        let success = operation.operate(&prefix, &parsed, &rootdir, tempdir_arc.path())?;
                        if success {
                            println!("{}", format!(">> {} succeeded{}", prefix, additional_message(&parsed)).blue());
                            Ok(true)
                        } else {
                            println!("{}", format!(">> {} failed{}", prefix, additional_message(&parsed)).magenta());
                            Ok(false)
                        }
                    } else {
                        Err(cause!(ErrorType::TempDirCreationError))
                    }
                }
            })
        }).collect();

        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    // Check producer errors and aggregate results
    match produce_results {
        Err(err) => Err(err),
        Ok(results) => {
            println!("{}", format!(">> All check tasks have done!\n").blue());
            if results.iter().all(|r| *r) {
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}

fn additional_message(parsed: &Parsed) -> String {
    match (&parsed.name, &parsed.dsc) {
        (Some(name), Some(dsc))     => format!(" ({}: {})", name, dsc),
        (Some(name), None)          => format!(" ({})",     name),
        (None,       Some(ref dsc)) => format!(" ({})",     dsc),
        (None,       None)          => "".to_owned(),
    }
}

fn create_parallel_fetch_key(parsed: &Parsed) -> String {
    // key for deduplication (stable string for Method)
    let method_label = match &parsed.mtd {
        Some(m) => match m {
            crate::common::Method::Shallow => "shallow",
            crate::common::Method::ShallowNoSparse => "shallow_no_sparse",
            crate::common::Method::Partial => "partial",
        },
        None => "default",
    };
    format!("{}|{}|{}", parsed.url, parsed.rev, method_label)
}

#[cfg(test)]
mod sequence_test;
