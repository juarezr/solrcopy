#![deny(warnings)]

#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate clap;
extern crate glob;
extern crate regex;
extern crate reqwest;
extern crate url;
extern crate zip;

mod args;
mod backup;
mod commit;
mod connection;
mod fails;
mod fetch;
mod helpers;
mod restore;
mod save;
mod steps;

use args::Arguments;
use fails::*;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    startup();

    let parsed = Arguments::parse_from_args()?;

    match parsed {
        Arguments::Backup(gets) => backup::backup_main(gets),
        Arguments::Restore(puts) => restore::restore_main(puts),
        Arguments::Commit(comt) => commit::commit_main(comt),
    }
}

fn startup() {
    #[cfg(feature = "artifacts")]
    Arguments::release_artifacts();
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
}

// endregion

// end of file
