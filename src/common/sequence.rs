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
    use std::sync::Mutex;
    use std::collections::HashMap;
    use std::sync::mpsc;

    let len = parsed.len();
    let operation = operation.clone();

    // Channel to send (prefix, parsed, Arc<TempDir>) from producers to consumer
    let (tx, rx) = mpsc::channel::<(String, Parsed, Arc<TempDir>)>();
    let dedup_map = Arc::new(Mutex::new(HashMap::<String, Arc<TempDir>>::new()));

    // Spawn producers to prepare tempdirs (deduplicating by url+rev+mtd) and send messages
    let produce_results: Vec<_> = std::thread::scope(|s| {
        let handles: Vec<_> = parsed.into_iter().enumerate().map(|(i, parsed)| {
            let tx = tx.clone();
            let dedup_map = dedup_map.clone();
            let rootdir = rootdir.clone();
            let operation = operation.clone();
            s.spawn(move || -> Result<bool, Cause<ErrorType>> {
                let prefix = format!("No.{i} ");
                println!("{}", format!(">> {}({}/{}) started{}", prefix, i + 1, len, additional_message(&parsed)).blue());

                // key for deduplication
                let key = format!("{}|{}|{:?}", parsed.url, parsed.rev, parsed.mtd);

                // Acquire lock and check or create TempDir
                let mut map = dedup_map.lock().map_err(|_| cause!(ErrorType::TempDirCreationError))?;
                if let Some(td) = map.get(&key) {
                    let td_arc = td.clone();
                    drop(map);
                    tx.send((prefix, parsed, td_arc)).map_err(|_| cause!(ErrorType::TempDirCreationError))?;
                    Ok(true)
                } else {
                    // Create tempdir and store as Arc
                    let tempdir = common::fetch::fetch_target_to_tempdir(&prefix, &parsed)?;
                    let td_arc = Arc::new(tempdir);
                    map.insert(key, td_arc.clone());
                    drop(map);
                    tx.send((prefix, parsed, td_arc)).map_err(|_| cause!(ErrorType::TempDirCreationError))?;
                    Ok(true)
                }
            })
        }).collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    // Check producer errors
    if let Some(err) = produce_results.into_iter().find(|r| matches!(r, Err(..))) {
        return err;
    }

    // All producers finished; drop the original sender so receiver iterator ends when all clones are dropped
    drop(tx);

    // Consume messages serially and run operation
    let mut overall = true;
    for (prefix, parsed, tempdir_arc) in rx {
        let success = operation.operate(&prefix, &parsed, &rootdir, &*tempdir_arc)?;
        if success {
            println!("{}", format!(">> {} succeeded{}", prefix, additional_message(&parsed)).blue());
        } else {
            println!("{}", format!(">> {} failed{}", prefix, additional_message(&parsed)).magenta());
            overall = false;
        }
        println!("");
    }

    println!("{}", format!(">> All check tasks have done!\n").blue());
    Ok(overall)
}

fn additional_message(parsed: &Parsed) -> String {
    match (&parsed.name, &parsed.dsc) {
        (Some(name), Some(dsc))     => format!(" ({}: {})", name, dsc),
        (Some(name), None)          => format!(" ({})",     name),
        (None,       Some(ref dsc)) => format!(" ({})",     dsc),
        (None,       None)          => "".to_owned(),
    }
}

#[cfg(test)]
mod sequence_test;
