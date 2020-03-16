# solrcopy

Command line tool for backup and restore of documents stored in cores of [Apache Solr](https://lucene.apache.org/solr/).

## Status

![build-test-and-lint](https://github.com/juarezr/solrcopy/workflows/build-test-and-lint/badge.svg)

- solrcopy backup/restore
  - Should work well in most common cases.
  - Works for me... :)
- Packaging:
  - Not started yet
- Check the issues in github
- Patches welcome!

## Usage

1. Use the command `solrcopy backup` for dumping documents from a Solr core into local zip files.
   1. Use the switch `--query` for filtering the documents extracted by using a [Solr](https://lucene.apache.org/solr/guide/8_4/the-standard-query-parser.html) [Query](https://lucene.apache.org/solr/guide/8_4/the-standard-query-parser.html)
   2. Use the switch `--order` for specifing the sorting of documents extracted.
   3. Use the switch `--limit` for restricting the number of documents extracted.
   4. Use the switch `--select` for restricting the columns extracted.
2. Use the command `solrcopy restore` for uploading the extracted documents from local zip files into the same Solr core or another with same field names as extracted.
   1. The documents are updated in the target core in the same format that they were extracted.
   2. The documents are inserted/updated based on their `uniqueKey` field defined in core.
   3. If you want to change the documents/columns use the swithes in `solrcopy backup` for extracting more than one slice of documents to be updated.
3. For reducing time, you can use the switches `--readers`  and `--writers` for executing operations in parallel.

## Invocation

``` text
$ solrcopy --help
solrcopy 0.4.0
Command line tool for backup and restore of documents stored in cores of Apache Solr.

Solrcopy is a command for doing backup and restore of documents stored on Solr cores. It let you filter docs by using a
expression, limit quantity, define order and desired columns to export. The data is stored as json inside local zip
files. It is agnostic to data format, content and storage place. Because of this data is restored exactly as extracted
and your responsible for extracting, storing and updating the correct data from and into correct cores.

USAGE:
    solrcopy <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    backup     Dumps documents from a Apache Solr core into local backup files
    commit     Perform a commit in the Solr core index for persisting documents in disk/memory
    help       Prints this message or the help of the given subcommand(s)
    restore    Restore documents from local backup files into a Apache Solr core
```

``` text
$ solrcopy help backup
solrcopy-backup 0.4.0
Dumps documents from a Apache Solr core into local backup files

USAGE:
    solrcopy backup [FLAGS] [OPTIONS] --from <core> --into </path/to/output> --url <localhost:8983/solr>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
        --verbose    Show details of the execution

OPTIONS:
    -d, --doc-count <quantity>                 Number of documents retrieved from solr in each reader step [default: 4k]
    -f, --from <core>                          Case sensitive name of the Solr core for extracting documents
    -i, --into </path/to/output>               Existing folder for writing the zip backup files containing the extracted
                                               documents [env: SOLR_COPY_DIR=]
    -l, --limit <quantity>                     Maximum number of documents for retrieving from the core (like 100M)
    -m, --max-files <quantity>                 Max number of files of documents stored in each zip file [default: 200]
    -o, --order <field1:asc field2:desc>...    Solr core fields names for sorting documents for retrieval
    -p, --prefix <name>                        Optional prefix for naming the zip backup files when storing documents
    -q, --query <f1:val1 AND f2:val2>          Solr Query for filtering which documents are retrieved
    -r, --readers <count>                      Number parallel threads reading documents from solr core [default: 1]
    -s, --select <field1 field2>...            Names of core fields retrieved in each document [default: all but _*]
    -u, --url <localhost:8983/solr>            Url pointing to the Solr cluster [env: SOLR_COPY_URL=]
    -w, --writers <count>                      Number parallel threads writing documents into zip archives [default: 1]

$ solrcopy backup --url http://localhost:8983/solr --from demo --query 'price:[1 TO 400] AND NOT popularity:10' --order price:desc weight:asc --limit 10000 --select id date name price weight popularity manu cat store features --into ./tmp
```

``` text
$ solrcopy help restore
solrcopy-restore 0.4.0
Restore documents from local backup files into a Apache Solr core

USAGE:
    solrcopy restore [FLAGS] [OPTIONS] --from </path/to/zips> --into <core> --url <localhost:8983/solr>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
        --verbose    Show details of the execution

OPTIONS:
    -c, --commit <mode>                Mode to perform commits of the index while updating documents in the core
                                       [default: none]  [possible values: none, soft, hard]
    -f, --from </path/to/zips>         Existing folder for reading the zip backup files containing documents [env:
                                       SOLR_COPY_DIR=]
    -i, --into <core>                  Case sensitive name of the Solr core to upload documents
    -p, --pattern <core*.zip>          Search pattern for matching names of the zip backup files
    -r, --readers <count>              Number parallel threads reading documents from solr core [default: 1]
    -u, --url <localhost:8983/solr>    Url pointing to the Solr cluster [env: SOLR_COPY_URL=]
    -w, --writers <count>              Number parallel threads writing documents into zip archives [default: 1]

$ solrcopy restore --url http://localhost:8983/solr  --from ./tmp --into target
```

## Known Issues

- Error extracting documents from a Solr cloud cluster with unbalanced shards:
  - Cause: In this case Cause: Solr reports diferent document count according to the instance is answering the query.
  - Fix: extract data pointing directly to the shard instance address, not for the cloud address.

## Related Projects

1. [solrbulk](https://github.com/miku/solrbulk)
2. [solrdump](https://github.com/ubleipzig/solrdump)

---

## Building

For compiling a version from source:

1. Install rust following the instructions on [https://rustup.rs](https://rustup.rs)
2. Build with the command: `cargo build --release`
3. Install locally with the command: `cargo install`

## Development

For setting up a development environment:

For using Visual Studio Code:

1. Install rust following the instructions on [https://rustup.rs](https://rustup.rs)
2. Install Visual Studio Code following the instructions on the microsoft [site](https://code.visualstudio.com/download)
3. Install the following extensions in VS Code:
   - vadimcn.vscode-lldb
   - rust-lang.rust
   - swellaby.vscode-rust-test-adapter

You can also use Intellij Idea, vim, emacs or you prefered IDE.

## Testing

For setting up a testing environment you will need:

1. A server instance of [Apache Solr](https://lucene.apache.org/solr/)
2. A **source** core with some documents for testing the `solrcopy backup` command.
3. A **target** core with same schema for testing the `solrcopy restore` command.
4. Setting the server address and core names for the `solrcopy` parameters in command line or IDE launch configuration.

### Use a existing server

 1. Select on your Solr server a existing **source** core or create a new one and fill with some documents.
 2. Clone a new **target** core with the same schema as the previous but without documents.

### Install a server in a docker container

#### Using docker compose

1. Install docker stable for your [platform](https://docs.docker.com/install/#supported-platforms)
2. Install docker compose for your [platform](https://docs.docker.com/compose/install/#install-compose)
3. Create the container and the cores for testing with the commands bellow.
4. Check the cores created in the admin ui at `http://localhost:8983/solr`

``` bash
# Create the container with a solr server with two cores: 'demo' and 'target'
$ docker-compose -f docker/docker-compose.yml up -d
```

#### Using only docker tools

1. Install docker stable for your [platform](https://docs.docker.com/install/#supported-platforms)
2. Pull the [latest](https://hub.docker.com/_/solr) [docker solr](https://github.com/docker-solr/docker-solr) image from Docker  Hub.
3. Create 2 cores for testing with the commands bellow.
4. Check the cores created in the admin ui at `http://localhost:8983/solr`

``` bash
$ cd docker
# Pull solr latest solr image from docker hub
$ docker pull solr:slim
...
# 1. Create a container running solr and after
# 2. Create the **source** core with the name 'demo'
# 3. Import some docs into the 'demo' core
$ docker run -d --name test-container -p 8983:8983 solr:slim solr-demo
...
# Create a empty **target** core named 'target'
$ docker exec -it test-container solr create_core -c target
```

### Setting up Visual Studio Code

1. Edit the settings file `.vscode/launch.json`
2. Change to your **solr address** in all the launch configurations:
    1. point parameter `--url` replacing `http://localhost:8983/solr`
    2. point parameter `--from` in `Launch-backup` configuration to your existing core name
    3. point parameter `--into` in `Launch-backup` configuration to your cloned core name
3. Change the others settings according to your existing core details:
   1. Set the following parameters for specifying a query to extract documents:
      - `--query`
      - `--order`
      - `--select`
      - `--batch`
      - `--limit`
   2. Check the [Solr Query](https://lucene.apache.org/solr/guide/8_4/the-standard-query-parser.html) docs for understading this parameters.
4. Test the parameters in Solr admin UI at your core in **solr address** (somenthing like: [http://localhost:8983/solr/#/corename](http://localhost:8983/solr/#/corename))

---
