#![deny(warnings)]
#![deny(anonymous_parameters)]
#![deny(bare_trait_objects)]
#![deny(elided_lifetimes_in_paths)]
#![deny(single_use_lifetimes)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unused_extern_crates)]
#![deny(unused_import_braces)]

// switches for develoment only

// #![allow(unused_variables)]
// #![allow(unused_imports)]
// #![allow(dead_code)]

// app depedencies

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

use clap::Parser;

use simplelog::{ColorChoice, CombinedLogger, Config, SharedLogger, TermLogger, WriteLogger};

pub use crate::args::{Cli, Commands};
use crate::fails::{throw, BoxedResult};

use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = Cli::parse_from_args()?;

    command_exec(parsed)
}

pub fn command_exec(parsed: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let args = &parsed.arguments;
    match args {
        Commands::Backup(get) => backup::backup_main(get),
        Commands::Restore(put) => restore::restore_main(put),
        Commands::Commit(cmd) => commit::commit_main(cmd),
        Commands::Delete(del) => delete::delete_main(del),
        Commands::Generate(cpl) => assets::gen_assets(cpl),
    }
}

// region Cli impl

impl Cli {
    pub fn parse_from_args() -> BoxedResult<Self> {
        let res = Self::parse();
        if let Err(msg) = res.arguments.validate() {
            throw(msg)?;
        }
        res.start_log()?;
        Ok(res)
    }

    fn start_log(&self) -> Result<(), Box<dyn std::error::Error>> {
        let opt = self.arguments.get_logging();

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

// endregion

#[cfg(test)]
#[cfg(feature = "testsolr")]
mod solr_tests {

    use crate::{args::Cli, command_exec};
    use clap::Parser;

    fn test_command_line_args_for(args: &[&str]) {
        let parsed = Cli::parse_from(args);

        let res = command_exec(parsed);

        assert_eq!(res.is_ok(), true);
    }

    fn get_solr_url() -> String {
        std::env::var("SOLRCOPY_URL").unwrap_or_else(|_| "http://localhost:8983/solr".into())
    }

    fn get_output_dir() -> String {
        std::env::var("SOLRCOPY_DIR").unwrap_or_else(|_| "target".into())
    }

    /// Run this command to test backup from a running Solr instance
    fn check_exec_backup(url:  &str , dir: &str) {

        let test_args = &["solrcopy", "backup", "--url", url, "--core", "demo", "--dir", dir];

        test_command_line_args_for(test_args);
    }

    /// Run this command to test backup from a running Solr instance
    fn check_exec_restore(url:  &str , dir: &str) {
        let test_args = &[
            "solrcopy", "restore", "--url", url, "--core", "target", "--search", "demo", "--dir",
            dir,
        ];
        test_command_line_args_for(test_args);
    }

    #[test]
    /// Run this command to test all solrcopy process on a running Solr instance
    fn check_exec_solrcopy_with_solr_running() {
        let (uri, out) = (get_solr_url(), get_output_dir());
        let (url, dir) = (uri.as_str(), out.as_str());

        check_exec_backup(url, dir);

        check_exec_restore(url, dir);
    }

    /// Run this command to test backup from a running Solr instance
    #[test]
    fn check_exec_commit() {
        let uri = get_solr_url();
        let url = uri.as_str();

        let test_args =
            &["solrcopy", "commit", "--url", url, "--core", "demo", "--log-level", "debug"];

        test_command_line_args_for(test_args);
    }

    /// Run this command to test delete all docs in the from a running Solr instance
    #[test]
    fn check_exec_delete() {
        let uri = get_solr_url();
        let url = uri.as_str();

        let test_args = &[
            "solrcopy",
            "delete",
            "--url",
            url,
            "--core",
            "films",
            "--query",
            "*:*",
            "--flush",
            "hard",
            "--log-level",
            "debug",
        ];

        test_command_line_args_for(test_args);
    }

    #[test]
    /// Run this command to test backup from a running Solr instance
    fn check_exec_generate() {
        let out = get_output_dir();
        let dir = out.as_str();

        let test_args = &["solrcopy", "generate", "--all", "--output-dir", dir];

        test_command_line_args_for(test_args);
    }
}

// end of file
