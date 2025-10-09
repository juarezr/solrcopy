#[cfg(test)]
/// Test against Solr instance running on localhost:8983 by default
mod testsolr {

    // region Helpers

    use crate::args::{Cli, SOLR_COPY_DIR, SOLR_COPY_URL};
    use chrono::offset::Local;
    use clap::Parser;
    use glob::glob;
    use std::fs::remove_file;

    fn test_command_line_args_for(args: &[&str]) {
        // Use the same args as solrcopy binary for testing
        let parsed = Cli::parse_from(args);
        execute_command_for(args, parsed);
    }

    #[cfg(not(feature = "testsolr"))]
    fn execute_command_for(args: &[&str], parsed: Cli) {
        assert_eq!(args.len() > 0, true);
        assert_eq!(parsed.arguments.validate(), Ok(()));
    }

    #[cfg(feature = "testsolr")]
    fn execute_command_for(args: &[&str], parsed: Cli) {
        // Execute the same way solrcopy would exec
        let res = crate::wrangle::command_exec(&parsed.arguments);
        // Display the error message if it failed
        if let Err(err) = res {
            let res = format!("Command: {}", args.join(" "));
            let msg = format!("Failure: {:?}", err);
            assert_eq!(res, msg, "\n   Tip: Check it the core has any documents indexed.");
        }
    }

    fn get_solr_url() -> String {
        std::env::var(SOLR_COPY_URL).unwrap_or_else(|_| "http://localhost:8983/solr".into())
    }

    fn get_output_dir() -> String {
        std::env::var(SOLR_COPY_DIR).unwrap_or_else(|_| "target".into())
    }

    fn cleanup_output_dir() {
        let listed = glob("target/demo*").unwrap();
        let found = listed.filter_map(Result::ok).collect::<Vec<_>>();
        for file in found {
            remove_file(file).unwrap();
        }
    }

    // endregion

    // region Tests

    /// Run this command to test backup from a running Solr instance
    fn check_exec_backup(url: &str, dir: &str, core: &str, comp: &str) {
        let test_args = &[
            "solrcopy",
            "backup",
            "--url",
            url,
            "--core",
            core,
            "--dir",
            dir,
            "--archive-compression",
            comp,
        ];

        test_command_line_args_for(test_args);
    }

    /// Run this command to test backup from a running Solr instance
    fn check_exec_restore(url: &str, dir: &str, core: &str) {
        let test_args = &[
            "solrcopy",
            "restore",
            "--url",
            url,
            "--core",
            core,
            "--search",
            "demo*.zip",
            "--dir",
            dir,
        ];
        test_command_line_args_for(test_args);
    }

    #[test]
    /// Run this command to test all solrcopy process on a running Solr instance
    fn check_exec_solrcopy_with_solr_running() {
        let (uri, out) = (get_solr_url(), get_output_dir());
        let (url, dir) = (uri.as_str(), out.as_str());

        cleanup_output_dir();

        check_exec_backup(url, dir, "demo", "zip");

        check_exec_delete(url, "target");

        check_exec_restore(url, dir, "target");

        check_exec_backup(url, dir, "demo", "zstd");
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

    /// Run this command to test backup from a running Solr instance
    #[test]
    fn check_exec_create() {
        let uri = get_solr_url();
        let url = uri.as_str();
        let now = Local::now().timestamp();
        let added = format!("added_{}", now);

        let test_args =
            &["solrcopy", "create", "--url", url, "--core", &added, "--log-level", "debug"];

        test_command_line_args_for(test_args);
    }

    /// Run this command to test delete all docs in the from a running Solr instance
    fn check_exec_delete(url: &str, core: &str) {
        let test_args = &[
            "solrcopy",
            "delete",
            "--url",
            url,
            "--core",
            core,
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

    // endregion
}
