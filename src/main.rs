// region Module and crate references

// region Strict Linting

#![deny(warnings)]
#![deny(anonymous_parameters)]
#![deny(bare_trait_objects)]
#![deny(elided_lifetimes_in_paths)]
#![deny(single_use_lifetimes)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unused_extern_crates)]
#![deny(unused_import_braces)]

// endregion

// region Switches for develoment only (do not commit enabled)

// #![allow(unused_variables)]
// #![allow(unused_imports)]
// #![allow(dead_code)]

// endregion

// region Imported Modules

#[macro_use]
extern crate lazy_static;

mod args;
mod assets;
mod backup;
mod bars;
mod commit;
mod connection;
mod delete;
#[macro_use]
mod fails;
mod fetch;
mod helpers;
mod ingest;
mod restore;
mod save;
mod state;
mod steps;
mod testsolr;

// endregion

// endregion

// region Main Entry Point

use crate::args::Cli;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = Cli::parse_from_args()?;

    wrangle::command_exec(parsed)
}

// endregion

// region Command line parsing

mod wrangle {

    use crate::args::{Cli, Commands, LoggingArgs};
    use crate::fails::{throw, BoxedResult};
    use crate::{assets, backup, commit, delete, restore};
    use clap::Parser;
    use simplelog::{ColorChoice, CombinedLogger, Config, SharedLogger, TermLogger, WriteLogger};
    use std::fs::File;

    pub(crate) fn command_exec(parsed: Cli) -> Result<(), Box<dyn std::error::Error>> {
        let args = &parsed.arguments;
        match args {
            Commands::Backup(get) => backup::backup_main(get),
            Commands::Restore(put) => restore::restore_main(put),
            Commands::Commit(cmd) => commit::commit_main(cmd),
            Commands::Delete(del) => delete::delete_main(del),
            Commands::Generate(cpl) => assets::gen_assets(cpl),
        }
    }

    impl Cli {
        pub(crate) fn parse_from_args() -> BoxedResult<Self> {
            let res = Self::parse();
            if let Err(msg) = res.arguments.validate() {
                throw(msg)?;
            }
            res.start_log()?;
            Ok(res)
        }

        fn start_log(&self) -> Result<(), Box<dyn std::error::Error>> {
            let opt: LoggingArgs = self.arguments.get_logging();

            let mut enabled: Vec<Box<dyn SharedLogger>> = Vec::new();
            if !opt.is_quiet() {
                enabled.push(TermLogger::new(
                    opt.log_level,
                    Config::default(),
                    opt.log_mode,
                    ColorChoice::Auto,
                ));
            }
            if let Some(filepath) = &opt.log_file_path {
                let file_to_log = File::create(filepath).unwrap();
                enabled.push(WriteLogger::new(opt.log_level, Config::default(), file_to_log));
            }
            CombinedLogger::init(enabled).unwrap();
            Ok(())
        }
    }
}

// endregion
