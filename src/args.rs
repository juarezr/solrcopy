// region depedencies

use regex::Regex;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use url::Url;

use crate::helpers::*;

// endregion

// region Order By

pub enum SortDirection {
    Asc,
    Desc,
}

impl fmt::Debug for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SortDirection::Asc => "asc",
                _ => "desc",
            }
        )
    }
}

pub struct SortField {
    pub field: String,
    pub direction: SortDirection,
}

impl fmt::Display for SortField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}%20{:?}", self.field, self.direction)
    }
}

impl fmt::Debug for SortField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{:?}", self.field, self.direction)
    }
}

impl FromStr for SortField {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("missing value".to_string())
        } else {
            lazy_static! {
                static ref REO: Regex = Regex::new("^(\\w+)(([:\\s=])(asc|desc))?$").unwrap();
            }
            match REO.captures(s) {
                None => Err(s.to_string()),
                Some(cap) => {
                    let sort_dir = if cap.get_as_str(4) == "desc" {
                        SortDirection::Desc
                    } else {
                        SortDirection::Asc
                    };
                    let sort_field = cap.get_as_str(1).to_string();
                    Ok(SortField {
                        field: sort_field,
                        direction: sort_dir,
                    })
                }
            }
        }
    }
}

// endregion

// region Cli structs

const SOLR_COPY_DIR: &str = "SOLR_COPY_DIR";
const SOLR_COPY_URL: &str = "SOLR_COPY_URL";

#[derive(StructOpt, Debug)]
pub struct Backup {
    /// Case sensitive name of the Solr core for extracting documents
    #[structopt(short, long, value_name = "core")]
    pub from: String,

    /// Solr Query for filtering which documents are retrieved
    #[structopt(short, long, value_name = "f1:val1 AND f2:val2")]
    pub query: Option<String>,

    /// Names of core fields retrieved in each document [default: all but _*]
    #[structopt(short, long, value_name = "field1 field2")]
    pub select: Vec<String>,

    /// Solr core fields names for sorting documents for retrieval
    #[structopt(short, long, value_name = "field1:asc field2:desc")]
    pub order: Vec<SortField>,

    /// Maximum number of documents for retrieving from the core (like 100M)
    #[structopt(short, long, parse(try_from_str = parse_quantity), min_values = 1, value_name = "quantity")]
    pub limit: Option<u64>,

    /// Existing folder for writing the zip backup files containing the extracted documents
    #[structopt(short, long, parse(from_os_str), env = SOLR_COPY_DIR, value_name = "/path/to/output")]
    pub into: PathBuf,

    /// Optional prefix for naming the zip backup files when storing documents
    #[structopt(short, long, parse(try_from_str = parse_file_prefix), value_name = "name")]
    pub prefix: Option<String>,

    #[structopt(flatten)]
    pub options: Options,

    /// Number of documents retrieved from solr in each reader step
    #[structopt(short, long, default_value = "4k", parse(try_from_str = parse_quantity), min_values = 1, value_name = "quantity")]
    pub batch: u64,
}

#[derive(StructOpt, PartialEq, Debug)]
/// Tells Solrt to performs a commit of the index while updating the core
pub enum CommitMode {
    /// Do not perform a commit
    None,
    /// Perform a soft commit to memory of the documents
    Soft,
    /// Perform a hard commit to disk of the documents (slow)
    Hard,
}

const COMMIT_VALUES: &[&str] = &["none", "soft", "hard"];

#[derive(StructOpt, Debug)]
pub struct Restore {
    /// Case sensitive name of the Solr core to upload documents
    #[structopt(short, long, value_name = "core")]
    pub into: String,

    /// Mode to perform commits of the index while updating documents in the core
    #[structopt(short, long, default_value = "none", possible_values = COMMIT_VALUES, value_name = "mode")]
    pub commit: CommitMode,

    /// Existing folder for reading the zip backup files containing documents
    #[structopt(short, long, parse(from_os_str), env = SOLR_COPY_DIR, value_name = "/path/to/zips")]
    pub from: PathBuf,

    /// Search pattern for matching names of the zip backup files
    #[structopt(short, long, value_name = "core*.zip")]
    pub pattern: Option<String>,

    #[structopt(flatten)]
    pub options: Options,
}

#[derive(StructOpt, Debug)]
pub struct Commit {
    /// Case sensitive name of the Solr core to perform the commit
    #[structopt(short, long, value_name = "core")]
    pub into: String,

    #[structopt(flatten)]
    pub options: Options,
}

#[derive(StructOpt, Debug)]
pub enum Arguments {
    /// Dumps documents from a Apache Solr core into local backup files
    Backup(Backup),
    /// Restore documents from local backup files into a Apache Solr core
    Restore(Restore),
    /// Perform a commit in the Solr core index for persisting documents in disk/memory
    Commit(Commit),
}

#[derive(StructOpt, Debug)]
/// Dumps and restores documents from a Apache Solr core into local backup files
pub struct Options {
    /// Url pointing to the Solr cluster
    #[structopt(short, long, env = SOLR_COPY_URL, parse(try_from_str = parse_solr_url), value_name = "localhost:8983/solr")]
    pub url: String,

    /// Show details of the execution
    #[structopt(long)]
    pub verbose: bool,
}

// endregion

// region Cli impl

fn parse_quantity(src: &str) -> Result<u64, String> {
    let norm = src
        .to_ascii_uppercase()
        .replace('K', "000")
        .replace('M', "000000")
        .replace('G', "000000000")
        .replace('T', "000000000000");

    let qt = norm.parse::<u64>();
    qt.or_else(|_| {
        Err(format!(
            "Wrong value: '{}'. Use numbers only, or suffix: K M G",
            src
        ))
    })
}

fn parse_solr_url(src: &str) -> Result<String, String> {
    let url2 = if src.starts_with_any(&["http://", "https://"]) {
        src.to_owned()
    } else {
        "http://".append(src)
    };
    let parsing = Url::parse(src);
    if let Err(reason) = parsing {
        return Err(format!("Error parsing Solr: {}", reason));
    }
    let parsed = parsing.unwrap();
    if parsed.scheme() != "http" {
        return Err(
            "Solr url scheme must be http or https as in: http:://server.domain:8983/solr"
                .to_string(),
        );
    }
    if parsed.query().is_some() {
        return Err("Solr url scheme must be a base url without query parameters as in: http:://server.domain:8983/solr".to_string());
    }
    if parsed.path_segments().is_none() {
        return Err(
            "Solr url path must be 'solr' as in: http:://server.domain:8983/solr".to_string(),
        );
    } else {
        let paths: Vec<&str> = parsed.path_segments().unwrap().collect();
        if paths.len() > 1 {
            return Err(
                "Solr url path must not include core name as in: http:://server.domain:8983/solr"
                    .to_string(),
            );
        }
    }
    Ok(url2)
}

fn parse_file_prefix(src: &str) -> Result<String, String> {
    lazy_static! {
        static ref REGFN: Regex = Regex::new("^(\\w+)$").unwrap();
    }
    match REGFN.get_group(src, 1) {
        None => Err(format!(
            "Wrong output filename: '{}'. Considere using letters and numbers.",
            src
        )),
        Some(group1) => Ok(group1.to_string()),
    }
}

impl Default for CommitMode {
    fn default() -> Self {
        CommitMode::None
    }
}

impl FromStr for CommitMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_ascii_lowercase();
        match lower.as_str() {
            "none" => Ok(CommitMode::None),
            "soft" => Ok(CommitMode::Soft),
            "hard" => Ok(CommitMode::Hard),
            _ => Err(format!("'{}' is not a valid value for CommitMode", s)),
        }
    }
}

impl CommitMode {
    pub fn as_param(&self, separator: &str) -> String {
        match self {
            CommitMode::None => EMPTY_STRING,
            CommitMode::Soft => separator.append("softCommit=true"),
            CommitMode::Hard => separator.append("commit=true"),
        }
    }
}

// endregion

#[cfg(test)]
pub mod tests {

    // region Mockup

    use crate::args::{Arguments, CommitMode};

    use structopt::StructOpt;

    impl Arguments {
        pub fn mockup_from(argument_list: &[&str]) {
            match Self::from_iter_safe(argument_list) {
                Ok(_) => panic!(
                    "Error parsing command line arguments: {}",
                    argument_list.join(" ")
                ),
                Err(_) => (),
            }
        }

        pub fn mockup_args_backup() -> Self {
            Self::from_iter(TEST_ARGS_BACKUP)
        }

        pub fn mockup_args_restore() -> Self {
            Self::from_iter(TEST_ARGS_RESTORE)
        }

        pub fn mockup_args_commit() -> Self {
            Self::from_iter(TEST_ARGS_COMMIT)
        }
    }

    pub const TEST_SELECT_FIELDS: &'static str = "id,date,vehiclePlate";

    const TEST_ARGS_HELP: &'static [&'static str] = &["solrcopy", "--help"];

    const TEST_ARGS_VERSION: &'static [&'static str] = &["solrcopy", "--version"];

    const TEST_ARGS_HELP_BACKUP: &'static [&'static str] = &["solrcopy", "help", "backup"];

    const TEST_ARGS_HELP_RESTORE: &'static [&'static str] = &["solrcopy", "help", "restore"];

    const TEST_ARGS_BACKUP: &'static [&'static str] = &[
        "solrcopy",
        "backup",
        "--url",
        "http://solr-server.com:8983/solr",
        "--from",
        "mileage",
        "--into",
        "./tmp",
        "--query",
        "ownerId:173826 AND periodCode:1",
        "--order",
        "date:asc",
        "id:desc",
        "vehiclePlate:asc",
        "--select",
        TEST_SELECT_FIELDS,
        "--prefix",
        "output_filename",
        "--limit",
        "42",
        "--batch",
        "5",
        "--verbose",
    ];

    const TEST_ARGS_RESTORE: &'static [&'static str] = &[
        "solrcopy",
        "restore",
        "--url",
        "http://solr-server.com:8983/solr",
        "--from",
        "./tmp",
        "--into",
        "target",
        "--pattern",
        "*.zip",
        "--commit",
        "soft",
        "--verbose",
    ];

    const TEST_ARGS_COMMIT: &'static [&'static str] = &[
        "solrcopy",
        "commit",
        "--url",
        "http://solr-server.com:8983/solr",
        "--into",
        "mileage",
        "--verbose",
    ];

    // endregion

    #[test]
    fn check_params_backup() {
        let parsed = Arguments::mockup_args_backup();
        match parsed {
            Arguments::Backup(get) => {
                assert_eq!(get.options.url, TEST_ARGS_BACKUP[3]);
                assert_eq!(get.from, TEST_ARGS_BACKUP[5]);
                assert_eq!(get.into.to_str().unwrap(), TEST_ARGS_BACKUP[7]);
                assert_eq!(get.query, Some(TEST_ARGS_BACKUP[9].to_string()));
                assert_eq!(get.limit, Some(42));
                assert_eq!(get.batch, 5);
                assert_eq!(get.options.verbose, true);
            }
            _ => panic!("command must be 'backup' !"),
        };
    }

    #[test]
    fn check_params_restore() {
        let parsed = Arguments::mockup_args_restore();
        match parsed {
            Arguments::Restore(put) => {
                assert_eq!(put.options.url, TEST_ARGS_RESTORE[3]);
                assert_eq!(put.from.to_str().unwrap(), TEST_ARGS_RESTORE[5]);
                assert_eq!(put.into, TEST_ARGS_RESTORE[7]);
                assert_eq!(put.pattern.unwrap(), TEST_ARGS_RESTORE[9]);
                assert_eq!(put.commit, CommitMode::Soft);
                assert_eq!(put.commit.as_param("?"), "?softCommit=true");
                assert_eq!(put.options.verbose, true);
            }
            _ => panic!("command must be 'restore' !"),
        };
    }

    #[test]
    fn check_params_commit() {
        let parsed = Arguments::mockup_args_commit();
        match parsed {
            Arguments::Commit(put) => {
                assert_eq!(put.options.url, TEST_ARGS_COMMIT[3]);
                assert_eq!(put.into, TEST_ARGS_COMMIT[5]);
                assert_eq!(put.options.verbose, true);
            }
            _ => panic!("command must be 'commit' !"),
        };
    }

    #[test]
    fn check_params_help() {
        Arguments::mockup_from(TEST_ARGS_HELP);
    }

    #[test]
    fn check_params_version() {
        Arguments::mockup_from(TEST_ARGS_VERSION);
    }

    #[test]
    fn check_params_get_help() {
        Arguments::mockup_from(TEST_ARGS_HELP_BACKUP);
    }

    #[test]
    fn check_params_put_help() {
        Arguments::mockup_from(TEST_ARGS_HELP_RESTORE);
    }
}

// end of file
