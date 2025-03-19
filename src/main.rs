// region Module and crate references

// region Strict Linting

#![deny(warnings)]
#![deny(anonymous_parameters)]
#![deny(bare_trait_objects)]
#![deny(elided_lifetimes_in_paths)]
#![deny(missing_debug_implementations)]
#![deny(single_use_lifetimes)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(unused_extern_crates)]
#![deny(unused_must_use)]
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
mod create;
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

    wrangle::command_exec(&parsed)
}

// endregion

// region Command line parsing

mod wrangle {

    use crate::args::{Cli, Commands};
    use crate::fails::{BoxedResult, throw};
    use crate::{assets, backup, commit, create, delete, restore};
    use clap::Parser;

    pub(crate) fn command_exec(args: &Commands) -> Result<(), Box<dyn std::error::Error>> {
        match args {
            Commands::Backup(get) => backup::backup_main(get),
            Commands::Restore(put) => restore::restore_main(put),
            Commands::Commit(cmd) => commit::commit_main(cmd),
            Commands::Delete(del) => delete::delete_main(del),
            Commands::Create(cre) => create::create_main(cre),
            Commands::Generate(cpl) => assets::gen_assets(cpl),
        }
    }

    impl Cli {
        pub(crate) fn parse_from_args() -> BoxedResult<Commands> {
            let parsed = Self::parse();
            let cmds = parsed.arguments;
            if let Err(msg) = cmds.validate() {
                throw(msg)?;
            }
            cmds.get_logging().start_log()?;
            Ok(cmds)
        }
    }
}

// endregion
