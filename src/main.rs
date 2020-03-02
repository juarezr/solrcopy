#![deny(warnings)]

#[macro_use]
extern crate lazy_static;

extern crate chrono;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = Arguments::parse_from_args()?;

    match parsed {
        Arguments::Backup(gets) => backup::backup_main(gets),
        Arguments::Restore(puts) => restore::restore_main(puts),
        Arguments::Commit(comt) => commit::commit_main(comt),
    }
}

// end of file
