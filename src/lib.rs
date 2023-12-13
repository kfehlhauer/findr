use crate::EntryType::*;
use clap::{Arg, ArgAction, Command};
use regex::Regex;
use std::error::Error;
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("findr")
        .version("0.1.0")
        .author("Kurt Fehlhauer")
        .about("Rust find")
        .arg(
            Arg::new("paths")
                .value_name("PATH")
                .help("Search paths [default: .]")
                .num_args(0..)
                .default_value("."),
        )
        .arg(
            Arg::new("names")
                .value_name("NAME")
                .help("Name")
                .short('n')
                .long("name")
                .value_parser(Regex::new)
                .action(ArgAction::Append)
                .num_args(0..),
        )
        .arg(
            Arg::new("type")
                .value_name("TYPE")
                .help("The type:")
                .short('t')
                .long("type")
                .value_parser(["d", "f", "l"])
                .num_args(0..),
        )
        .get_matches();

    Ok(Config {
        paths: matches
            .get_many("paths")
            .unwrap_or_default()
            .cloned()
            .collect(),
        names: matches
            .get_many("names")
            .unwrap_or_default()
            .cloned()
            .collect(),
        entry_types: if let Some(types) = matches.get_many::<String>("type") {
            types
                .filter_map(|type_str| match type_str.to_uppercase().as_str() {
                    "D" => Some(EntryType::Dir),
                    "F" => Some(EntryType::File),
                    "L" => Some(EntryType::Link),
                    _ => None,
                })
                .collect()
        } else {
            vec![]
        },
    })
}

fn filter_types(entry: &walkdir::DirEntry, entry_type: &EntryType) -> bool {
    match entry_type {
        EntryType::Dir => entry.path().is_dir(),
        EntryType::File => entry.path().is_file() && !entry.path().is_symlink(),
        EntryType::Link => entry.path().is_symlink(),
    }
}

pub fn run(config: Config) -> MyResult<()> {
    for path in config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprintln!("{}", e),
                Ok(entry) => {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if config.names.is_empty()
                            || config.names.iter().any(|regex| regex.is_match(file_name))
                        {
                            if !config.entry_types.is_empty() {
                                for entry_type in &config.entry_types {
                                    if filter_types(&entry, &entry_type) {
                                        println!("{}", entry.path().display());
                                    }
                                }
                            } else {
                                println!("{}", entry.path().display());
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
