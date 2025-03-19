# solrcopy

Command line tool for backup and restore of documents stored in cores of [Apache Solr](https://lucene.apache.org/solr/).

## Status

![build-test-and-lint](https://github.com/juarezr/solrcopy/workflows/build-test-and-lint/badge.svg)

[![Coverage Status](https://coveralls.io/repos/github/juarezr/solrcopy/badge.svg?branch=master)](https://coveralls.io/github/juarezr/solrcopy?branch=master)

- solrcopy backup/restore
  - Should work well in most common cases.
  - Works for me... :)
- Check the issues in github
- Patches welcome!

<!-- markdownlint <!-- markdownlint-disable-next-line no-inline-html --> -->
[<img alt="Send some cookies" src="http://img.shields.io/liberapay/receives/juarezr.svg?label=Send%20some%20cookies&logo=liberapay">](https://liberapay.com/juarezr/donate)

## Usage

1. Use the command `solrcopy backup` for dumping documents from a Solr core into local zip files.
   1. Use the switch `--query` for filtering the documents extracted by using a [Solr](https://lucene.apache.org/solr/guide/8_4/the-standard-query-parser.html) [Query](https://lucene.apache.org/solr/guide/8_4/the-standard-query-parser.html)
   2. Use the switch `--order` for specifying the sorting of documents extracted.
   3. Use the switches `--limit` and `--skip` for restricting the number of documents extracted.
   4. Use the switch `--select` for restricting the columns extracted.
2. Use the command `solrcopy restore` for uploading the extracted documents from local zip files into the same Solr core or another with same field names as extracted.
   1. The documents are updated in the target core in the same format that they were extracted.
   2. The documents are inserted/updated based on their `uniqueKey` field defined in core.
   3. If you want to change the documents/columns use the switches in `solrcopy backup` for extracting more than one slice of documents to be updated.

### Huge cores

Extracting and updating documents in huge cores can be challenging. It can take too much time and can fail any time.

Bellow some tricks for dealing with such cores:

1. For reducing time, you can use the switches `--readers`  and `--writers` for executing operations in parallel.
2. When the number of docs to extract is huge, `backup` subcommand tend to slow as times goes and eventually fails. This is because Solr is suffers to get docs batches with hight skip/start parameters. For dealing with this:
   1. Use the parameters `--iterate-by`n `between` and `--step`for iterating through parameter `--query` with variables `{begin}` and `{end}`.
   2. This way it will iterate and restrict by hour, day, range the docs being downloaded.
   3. For example: `--query 'date:[{begin} TO {end}]' --iterate-by day --between '2020-04-01' '2020-04-30T23:59:59'`
3. Use the parameter `--param shards=shard1` for copying by each shard by name in `backkup`subcommand.
4. Use the parameter `--delay` for avoiding to overload the Solr server.

## Invocation

``` text
$ solrcopy --help
Command line tool for backup and restore of documents stored in cores of Apache Solr.

Solrcopy is a command for doing backup and restore of documents stored on Solr cores. It let you filter docs by using a expression, limit quantity, define order and desired columns to export. The data is stored as json inside local zip files. It is agnostic to data format, content and storage place. Because of this data is restored exactly as extracted and your responsible for extracting, storing and updating the correct data from and into correct cores.

Usage: solrcopy <COMMAND>

Commands:
  backup    Dumps documents from a Apache Solr core into local backup files
  restore   Restore documents from local backup files into a Apache Solr core
  commit    Perform a commit in the Solr core index for persisting documents in disk/memory
  delete    Removes documents from the Solr core definitively
  generate  Generates man page and completion scripts for different shells
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

``` text
$ solrcopy backup --help
Dumps documents from a Apache Solr core into local backup files

Usage: solrcopy backup [OPTIONS] --url <localhost:8983/solr> --core <core> --dir </path/to/output>

Options:
  -u, --url <localhost:8983/solr> [env: SOLR_COPY_URL=]
          Url pointing to the Solr cluster

  -c, --core <core>
          Case sensitive name of the core in the Solr server

  -d, --dir </path/to/output> [env: SOLR_COPY_DIR=]
          Existing folder where the zip backup files containing the extracted documents are stored

  -q, --query <'f1:vl1 AND f2:vl2'>
          Solr Query param 'q' for filtering which documents are retrieved See: https://lucene.apache.org/solr/guide/6_6/the-standard-query-parser.html

  -o, --order <f1:asc,f2:desc,...>
          Solr core fields names for sorting documents for retrieval

  -k, --skip <quantity>
          Skip this quantity of documents in the Solr Query [default: 0]

  -l, --limit <quantity>
          Maximum quantity of documents for retrieving from the core (like 100M)

  -s, --select <field1,field2,...>
          Names of core fields retrieved in each document [default: all but _*]

  -i, --iterate-by <mode>
          Slice the queries by using the variables {begin} and {end} for iterating in `--query` Used in bigger solr cores with huge number of docs because querying the end of docs is expensive and fails frequently [default: day]

          Possible values:
          - none
          - minute: Break the query in slices by a first ordered date field repeating between {begin} and {end} in the query parameters
          - hour
          - day
          - range:  Break the query in slices by a first ordered integer field repeating between {begin} and {end} in the query parameters

  -b, --between <begin> <end> <begin> <end>
          The range of dates/numbers for iterating the queries throught slices. Requires that the query parameter contains the variables {begin} and {end} for creating the slices. Use numbers or dates in ISO 8601 format (yyyy-mm-ddTHH:MM:SS)

      --step <num>
          Number to increment each step in iterative mode [default: 1]
          
  -p, --params <useParams=mypars>
          Extra parameter for Solr Update Handler. See: https://lucene.apache.org/solr/guide/transforming-and-indexing-custom-json.html

  -m, --max-errors <count>
          How many times should continue on source document errors [default: 0]

      --delay-before <time>
          Delay before any processing in solr server. Format as: 30s, 15min, 1h

      --delay-per-request <time>
          Delay between each http operations in solr server. Format as: 3s, 500ms, 1min

      --delay-after <time>
          Delay after all processing. Usefull for letting Solr breath

      --num-docs <quantity>
          Number of documents to retrieve from solr in each reader step [default: 4k]

      --archive-files <quantity>
          Max number of files of documents stored in each zip file [default: 40]

      --zip-prefix <name>
          Optional prefix for naming the zip backup files when storing documents

      --workaround-shards <count>
          Use only when your Solr Cloud returns a distinct count of docs for some queries in a row. This may be caused by replication problems between cluster nodes of shard replicas of a core. Response with 'num_found' bellow the greatest value are ignored for getting all possible docs. Use with `--params shards=shard_name` for retrieving all docs for each shard of the core

  -r, --readers <count>
          Number parallel threads exchanging documents with the solr core [default: 1]

  -w, --writers <count>
          Number parallel threads syncing documents with the zip archives [default: 1]

      --log-level <level>
          What level of detail should print messages [default: info]

      --log-mode <mode>
          Terminal output to print messages [default: mixed]
          

      --log-file-path <path>
          Write messages to a local file

      --log-file-level <level>
          What level of detail should write messages to the file [default: debug]

  -h, --help
          Print help (see a summary with '-h')

$ solrcopy backup --url http://localhost:8983/solr --core demo --query 'price:[1 TO 400] AND NOT popularity:10' --order price:desc,weight:asc --limit 10000 --select id,date,name,price,weight,popularity,manu,cat,store,features --dir ./tmp
```

``` text
$ solrcopy restore --help
Restore documents from local backup files into a Apache Solr core

Usage: solrcopy restore [OPTIONS] --url <localhost:8983/solr> --core <core> --dir </path/to/output>

Options:
  -u, --url <localhost:8983/solr>  Url pointing to the Solr cluster [env: SOLR_COPY_URL=]
  -c, --core <core>                Case sensitive name of the core in the Solr server
  -d, --dir </path/to/output>      Existing folder where the zip backup files containing the extracted documents are stored [env: SOLR_COPY_DIR=]
  -f, --flush <mode>               Mode to perform commits of the documents transaction log while updating the core [possible values: none, soft, hard, <interval>] [default: hard]
      --no-final-commit            Do not perform a final hard commit before finishing
      --disable-replication        Disable core replication at start and enable again at end
  -p, --params <useParams=mypars>  Extra parameter for Solr Update Handler. See: https://lucene.apache.org/solr/guide/transforming-and-indexing-custom-json.html
  -m, --max-errors <count>         How many times should continue on source document errors [default: 0]
      --delay-before <time>        Delay before any processing in solr server. Format as: 30s, 15min, 1h
      --delay-per-request <time>   Delay between each http operations in solr server. Format as: 3s, 500ms, 1min
      --delay-after <time>         Delay after all processing. Usefull for letting Solr breath
  -s, --search <core*.zip>         Search pattern for matching names of the zip backup files
      --order <asc | desc>         Optional order for searching the zip archives
  -r, --readers <count>            Number parallel threads exchanging documents with the solr core [default: 1]
  -w, --writers <count>            Number parallel threads syncing documents with the zip archives [default: 1]
      --log-level <level>          What level of detail should print messages [default: info]
      --log-mode <mode>            Terminal output to print messages [default: mixed]
      --log-file-path <path>       Write messages to a local file
      --log-file-level <level>     What level of detail should write messages to the file [default: debug]
  -h, --help                       Print help

$ solrcopy restore --url http://localhost:8983/solr  --dir ./tmp --core demo
```

``` text
$ solrcopy delete --help
Removes documents from the Solr core definitively

Usage: solrcopy delete [OPTIONS] --query <f1:val1 AND f2:val2> --url <localhost:8983/solr> --core <core>

Options:
  -u, --url <localhost:8983/solr>    Url pointing to the Solr cluster [env: SOLR_COPY_URL=]
  -c, --core <core>                  Case sensitive name of the core in the Solr server
  -q, --query <f1:val1 AND f2:val2>  Solr Query for filtering which documents are removed in the core. 
                                     Use '*:*' for excluding all documents in the core. There are no way of recovering excluded docs.
                                     Use with caution and check twice
  -f, --flush <mode>                 Wether to perform a commits of transaction log after removing the documents [default: soft]
      --log-level <level>            What level of detail should print messages [default: info]
      --log-mode <mode>              Terminal output to print messages [default: mixed]
      --log-file-path <path>         Write messages to a local file
      --log-file-level <level>       What level of detail should write messages to the file [default: debug]
  -h, --help                         Print help

$ solrcopy delete --url http://localhost:8983/solr --core demo --query '*:*'
```

``` text
$ solrcopy commit --help
Perform a commit in the Solr core index for persisting documents in disk/memory

Usage: solrcopy commit [OPTIONS] --url <localhost:8983/solr> --core <core>

Options:
  -u, --url <localhost:8983/solr>  Url pointing to the Solr cluster [env: SOLR_COPY_URL=]
  -c, --core <core>                Case sensitive name of the core in the Solr server
      --log-level <level>          What level of detail should print messages [default: info]
      --log-mode <mode>            Terminal output to print messages [default: mixed]
      --log-file-path <path>       Write messages to a local file
      --log-file-level <level>     What level of detail should write messages to the file [default: debug]
  -h, --help                       Print help

$ solrcopy commit --url http://localhost:8983/solr --core demo
```

## Known Issues

- Error extracting documents from a Solr cloud cluster with corrupted shards or unreplicated replicas:
  - Cause: In this case Cause: Solr reports diferent document count each time is answering the query.
  - Fix: extract data pointing directly to the shard instance address, not for the cloud address.
  - Also can use custom params to solr as `--params timeAllowed=15000&segmentTerminatedEarly=false&cache=false&shards=shard1`

## Related

1. [solrbulk](https://github.com/miku/solrbulk)
2. [solrdump](https://github.com/ubleipzig/solrdump)
3. [Solr documentation of backup/restore](https://lucene.apache.org/solr/guide/6_6/making-and-restoring-backups.html)

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

Check the Solr docker [documentation](https://solr.apache.org/guide/solr/latest/deployment-guide/solr-in-docker.html) for help in how to create a Solr container.

#### Using docker compose

1. Install [docker stable](https://docs.docker.com/get-started/get-docker/) for your platform
2. Create the container and the cores for testing with the commands bellow.
3. Check the cores created in the admin ui at `http://localhost:8983/solr`

``` bash
# This command creates the container with a solr server with two cores: 'demo' and 'target'
$ docker compose -f docker/docker-compose.yml up -d
# Run this command to insert some data into the cores
$ docker compose exec solr solr-ingest-all
# Run this command to test backup
$ cargo run -- backup --url http://localhost:8983/solr --core demo --dir $PWD
# Run this command to test restoring the backukp data into a existing empty core
$ cargo run -- restore --url http://localhost:8983/solr --search demo --core target --dir $PWD
```

#### Using only docker tools

Its possible to create the solr container using just docker instead of docker compoose.

Follow these instructions if you'd rather prefer this way:

``` bash
$ cd docker
# Pull solr latest solr image from docker hub
$ docker pull solr:slim
...
# 1. Create a container running solr and after
# 2. Create the **source** core with the name 'demo'
# 3. Import some docs into the 'demo' core
$ docker run -d --name solr4test -p 8983:8983 solr:slim solr-demo
...
# Create a empty **target** core named 'target'
$ docker exec -it solr4test solr create_core -c target
```

### Developing in Visual Studio Code

There are some pre-configured launch configurations in this repository for debugging
solrcopy.

1. Start the SOLR docker container with the procedures above.
2. Run Solrcopy using one of the predefined lauch configuration.
    1. You will be asked for the program argumentls like:
        1. SolrURL
        2. Query
3. You can also edit the settings file `.vscode/launch.json` if you'd rather prefer:
   1. Set the following parameters for specifying a query to extract documents:
      - `--query`
      - `--order`
      - `--select`
      - `--batch`
      - `--skip`
      - `--limit`
   2. Check the [Solr Query](https://lucene.apache.org/solr/guide/8_4/the-standard-query-parser.html) docs for understanding this parameters.
4. You can also run any query in [Solr admin UI](http://localhost:8983/solr/#/demo)

---
