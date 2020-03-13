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

// TODO : hancle Ctrl + C and finish zip archives

// TODO: Cleanup
//
// #![deny(box_pointers)]
// #![deny(unused_results)]

// switches for develoment only
//
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
mod fails;
mod fetch;
mod helpers;
mod ingest;
mod restore;
mod save;
mod steps;

use structopt::StructOpt;

use args::Arguments;
use fails::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = Arguments::parse_from_args()?;
    parsed.start_log();

    match parsed {
        Arguments::Backup(gets) => backup::backup_main(gets),
        Arguments::Restore(puts) => restore::restore_main(puts),
        Arguments::Commit(comt) => commit::commit_main(comt),
    }
}

// region Cli impl

impl Arguments {
    pub fn parse_from_args() -> Result<Self, BoxedError> {
        let res = Self::from_args();
        res.check_dir()?;
        Ok(res)
    }

    pub fn check_dir(&self) -> Result<(), BoxedError> {
        let dir = match &self {
            Self::Backup(get) => &get.into,
            Self::Restore(put) => &put.from,
            Self::Commit(_) => return Ok(()),
        };
        if !dir.exists() {
            throw(format!("Missing folder of zip backup files: {:?}", dir))?
        }
        Ok(())
    }

    fn start_log(&self) {
        let verbose = match &self {
            Self::Backup(get) => get.options.verbose,
            Self::Restore(put) => put.options.verbose,
            Self::Commit(com) => com.options.verbose,
        };
        if verbose {
            env_logger::builder().filter_level(log::LevelFilter::Debug).init();
        } else {
            env_logger::builder().filter_level(log::LevelFilter::Info).init();
        }
    }
}

// endregion

// end of file
