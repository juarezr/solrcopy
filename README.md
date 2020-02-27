# solrcopy

Command line tool for backup and restore of information stored in cores of Apache Solr.

## Usage

1. Use the command `solrcopy backup` for dumping data/records from a Solr core into local zip files.
   1. Use the switch `--filter` for filtering the rows extracted by using a [Solr](https://lucene.apache.org/solr/guide/8_4/the-standard-query-parser.html) [Query](https://lucene.apache.org/solr/guide/8_4/the-standard-query-parser.html)
   2. Use the switch `--order` for specifing the sorting of rows extracted.
   3. Use the switch `--limit` for restricting the number of rows extracted.
   4. Use the switch `--select` for restricting the columns extracted.
2. Use the command `solrcopy restore` for uploading the extracted data/records from local zip files into the same Solr core or another with same field names as extracted.
   1. The rows are updated in the target core in the same format that they were extracted.
   2. The rows are inserted/updated based on their `uniqueKey` field defined in core.
   3. If you want to change the rows/columns use the swithes in `solrcopy backup` for extracting more than one slice of records to be updated.

## Invocation

``` bash
$ solrcopy --help
solrcopy 0.2.1

USAGE:
    solrcopy <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    backup     Dumps records from a Apache Solr core into local backup files
    help       Prints this message or the help of the given subcommand(s)
    restore    Restore records from local backup files into a Apache Solr core
```

``` bash
$ solrcopy help backup
solrcopy-backup 0.2.1
Dumps records from a Apache Solr core into local backup files

USAGE:
    solrcopy backup [FLAGS] [OPTIONS] --from <from> --into <into> --url <url>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
        --verbose    Show details of the execution

OPTIONS:
    -b, --batch <batch>         Number of records for reading from solr in each step [default: 4096]
    -w, --where <filter>        Solr Query filter for filtering returned records
    -f, --from <from>           Case sensitive name of the Solr core for extracting records
    -i, --into <into>           Existing folder for writing the dump files [env: SOLRDUMP_DIR=]
    -l, --limit <limit>         Maximum number of records for retrieving from the core
    -n, --name <name>           Name for writing backup zip files
    -o, --order <order>...      Solr core fields names for sorting records for retrieval (like: field1:desc)
    -s, --select <select>...    Solr core fields names for restricting columns for retrieval
    -u, --url <url>             Url pointing to the Solr base address like: http://solr-server:8983/solr [env:
                                SOLR_URL=]

$ solrcopy backup --url http://my-solr-server.com::8983/sol --from my-solr-core --where 'field1:123 AND field2:456' --order id:asc date:asc --limit 10000 --select id date name price otherfield --into ./my-core-folder
```

``` bash
$ solrcopy help restore
solrcopy-restore 0.2.1
Restore records from local backup files into a Apache Solr core

USAGE:
    solrcopy restore [FLAGS] [OPTIONS] --from <from> --into <into> --url <url>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
        --verbose    Show details of the execution

OPTIONS:
    -f, --from <from>          Existing folder for searching and reading the zip backup files [env: SOLRDUMP_DIR=]
    -i, --into <into>          Case sensitive name of the Solr core to upload records/data
    -p, --pattern <pattern>    Pattern for matching backup zip files in `from` folder for restoring
    -u, --url <url>            Url pointing to the Solr base address like: http://solr-server:8983/solr [env: SOLR_URL=]

$ solrcopy restore --url http://my-solr-server.com::8983/sol  --from ./my-core-folder --into my-solr-core
```

## Status

![Rust](https://github.com/juarezr/solrcopy/workflows/Rust/badge.svg)

- solrcopy backup/restore
  - Kind of working
  - Needs finishing some `TODO`
  - Lightly tested
- Packaging:
  - Not started yet
- Check the issues in github
- Patches welcome!

## Known Issues

- Error extracting rows from a Solr cloud cluster with unbalanced shards:
  - Cause: In this case Cause: Solr reports diferent row count according to the instance is answering the query.
  - Fix: extract data pointing directly to the shard instance address, not for the cloud address.

## Development

For setup of a development:

1. Install rust following the instructions on [https://rustup.rs](https://rustup.rs)
2. Install Visual Studio Code following the instructions on the microsoft [site](https://code.visualstudio.com/download)
3. Install the following extensions in VS Code:
   - vadimcn.vscode-lldb
   - rust-lang.rust
   - swellaby.vscode-rust-test-adapter

## Related

1. [solrbulk](https://github.com/miku/solrbulk)
2. [solrdump](https://github.com/ubleipzig/solrdump)
