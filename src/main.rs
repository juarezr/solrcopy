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
    if parsed.verbose {
        // TODO: use a logger and combine with --verbose
        println!("  {:?}", parsed);
    }

    backup::backup_main(parsed)
}

// end of file
