# solrbulk

Tools for backup and restore of information stored in cores of Apache Solr

## Status

![Rust](https://github.com/juarezr/solrbulk/workflows/Rust/badge.svg)

 - Work in Progress
 - Backup:
    * Working
	* Needs finishing some `TODO`
 - Restore:
     * Not started yet
	 * Alternatively could do: unzip archive.zip | curl http://solr.server:8193/update
 - Patches welcome!
	 
## Development

For setup of a development:

1. Install rust following the instructions on [https://rustup.rs](https://rustup.rs)
2. Install Visual Studio Code following the instructions on the microsoft [site](https://code.visualstudio.com/download)
3. Install the following extensions in VS Code:
  - vadimcn.vscode-lldb
  - rust-lang.rust
  - swellaby.vscode-rust-test-adapter
