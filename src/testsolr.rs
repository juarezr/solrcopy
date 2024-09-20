// region Testing against Solr server

#[cfg(test)]
// #[cfg(feature = "testsolr")]
/// Test against Solr instance running on localhost:8983 by default
mod testsolr {

    use crate::args::{Cli, SOLR_COPY_DIR, SOLR_COPY_URL};
    use crate::wrangle::command_exec;
    use clap::Parser;

    fn test_command_line_args_for(args: &[&str]) {
        // Use the same args as solrcopy binary for testing
        let parsed = Cli::parse_from(args);

        // Execute the same way solrcopy would exec
        let res = command_exec(parsed);

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

    /// Run this command to test backup from a running Solr instance
    fn check_exec_backup(url: &str, dir: &str) {
        let test_args = &["solrcopy", "backup", "--url", url, "--core", "demo", "--dir", dir];

        test_command_line_args_for(test_args);
    }

    /// Run this command to test backup from a running Solr instance
    fn check_exec_restore(url: &str, dir: &str) {
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

        check_exec_delete(url);
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
    fn check_exec_delete(url: &str) {
        let test_args = &[
            "solrcopy",
            "delete",
            "--url",
            url,
            "--core",
            "demo",
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

// endregion
