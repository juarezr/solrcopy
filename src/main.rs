#![deny(warnings)] 

#[macro_use] 
extern crate lazy_static;

extern crate regex;
extern crate url;
extern crate reqwest;
extern crate zip;
extern crate chrono;

mod fails;
mod helpers;
mod args;
mod steps;
mod fetch;
mod save;
mod backup;

use args::Arguments;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let parsed = Arguments::parse_from_args()?;

    match parsed {
        Arguments::Backup(gets) => backup::backup_main(gets),
        Arguments::Restore(_puts) => unimplemented!(),
    }
}

// end of file
