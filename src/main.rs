#![deny(warnings)]
#![deny(anonymous_parameters)]
#![deny(bare_trait_objects)]
#![deny(elided_lifetimes_in_paths)]
#![deny(single_use_lifetimes)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unused_extern_crates)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]

// TODO: Cleanup
//
// #![deny(box_pointers)]
// #![deny(unused_results)]

// switches for develoment only

// #![allow(unused_variables)]
// #![allow(unused_imports)]
// #![allow(dead_code)]
#[macro_use]
extern crate lazy_static;

mod args;
mod backup;
mod bars;
mod commit;
mod connection;
mod delete;
mod fails;
mod fetch;
mod helpers;
mod ingest;
mod restore;
mod save;
mod state;
mod steps;

use simplelog::{
    CombinedLogger, Config, LevelFilter, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};
use structopt::StructOpt;

use crate::args::{Arguments, Backup, CommonArgs, Restore};
use crate::fails::{throw, BoxedError, BoxedResult};

use std::fs::File;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = Arguments::parse_from_args()?;
    parsed.start_log()?;

    match parsed {
        Arguments::Backup(get) => backup::backup_main(get),
        Arguments::Restore(put) => restore::restore_main(put),
        Arguments::Commit(cmd) => commit::commit_main(cmd),
        Arguments::Delete(del) => delete::delete_main(del),
    }
}

// region Cli impl

impl Arguments {
    pub fn parse_from_args() -> BoxedResult<Self> {
        let res = Self::from_args();
        res.validate()?;
        Ok(res)
    }

    pub fn validate(&self) -> BoxedError {
        match self {
            Self::Backup(get) => get.validate(),
            Self::Restore(put) => put.validate(),
            Self::Commit(_) | Self::Delete(_) => Ok(()),
        }
    }

    pub fn get_options(&self) -> &CommonArgs {
        match &self {
            Self::Backup(get) => &get.options,
            Self::Restore(put) => &put.options,
            Self::Commit(com) => &com.options,
            Self::Delete(del) => &del.options,
        }
    }

    fn start_log(&self) -> Result<(), Box<dyn std::error::Error>> {
        let options = self.get_options();

        let mut enabled: Vec<Box<dyn SharedLogger>> = Vec::new();
        if !options.is_quiet() {
            let level = Self::parse_level_filter(options.log_level.as_str())?;
            let mode = Self::parse_term_mode(options.log_mode.as_str())?;
            enabled.push(TermLogger::new(level, Config::default(), mode).unwrap());
        }
        if let Some(filepath) = &options.log_file_path {
            let level2 = Self::parse_level_filter(options.log_file_level.as_str())?;
            let file_to_log = File::create(filepath).unwrap();
            enabled.push(WriteLogger::new(level2, Config::default(), file_to_log));
        }
        CombinedLogger::init(enabled).unwrap();
        Ok(())
    }

    fn parse_level_filter(s: &str) -> BoxedResult<LevelFilter> {
        match LevelFilter::from_str(s) {
            Ok(res) => Ok(res),
            Err(_) => throw(format!("'{}'. [alowed: off, error, warn, info, debug, trace]", s)),
        }
    }

    fn parse_term_mode(mode: &str) -> BoxedResult<TerminalMode> {
        let mode_str = mode.to_ascii_lowercase();
        match mode_str.as_ref() {
            "stdout" => Ok(TerminalMode::Stdout),
            "stderr" => Ok(TerminalMode::Stderr),
            "mixed" => Ok(TerminalMode::Mixed),
            _ => throw(format!("Unknown terminal mode: {}", mode_str)),
        }
    }
}

pub trait Validation {
    fn validate(&self) -> BoxedError {
        Ok(())
    }
}

impl Validation for Backup {
    fn validate(&self) -> BoxedError {
        assert_dir_exists(&self.transfer.dir)
    }
}

impl Validation for Restore {
    fn validate(&self) -> BoxedError {
        assert_dir_exists(&self.transfer.dir)
    }
}

fn assert_dir_exists(dir: &std::path::PathBuf) -> BoxedError {
    if !dir.exists() {
        throw(format!("Missing folder of zip backup files: {:?}", dir))?;
    }
    Ok(())
}

// endregion

// end of file
