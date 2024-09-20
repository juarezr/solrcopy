use crate::helpers::{CapturesHelpers, RegexHelpers, StringHelpers, EMPTY_STR, EMPTY_STRING};
use clap::builder::styling::{AnsiColor as Ansi, Styles};
use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use regex::Regex;
use simplelog::{LevelFilter, TerminalMode};
use std::{fmt, path::Path, path::PathBuf, str::FromStr};
use url::Url;

// #region Cli arguments

// #region Cli structs

const STYLES: Styles = Styles::styled()
    .usage(Ansi::BrightYellow.on_default().bold())
    .header(Ansi::BrightYellow.on_default().bold())
    .literal(Ansi::BrightBlue.on_default())
    .placeholder(Ansi::BrightMagenta.on_default())
    .error(Ansi::BrightRed.on_default())
    .valid(Ansi::BrightGreen.on_default())
    .invalid(Ansi::BrightRed.on_default());

/// Command line tool for backup and restore of documents stored in cores of Apache Solr.
///
/// Solrcopy is a command for doing backup and restore of documents stored on Solr cores.
/// It let you filter docs by using a expression, limit quantity, define order and desired
/// columns to export. The data is stored as json inside local zip files. It is agnostic
/// to data format, content and storage place. Because of this data is restored exactly
/// as extracted and your responsible for extracting, storing and updating the correct data
/// from and into correct cores.
#[derive(Parser, Debug)]
#[command(author, version, about, version, arg_required_else_help = true, styles = STYLES)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub arguments: Commands,
}

#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Commands {
    /// Dumps documents from a Apache Solr core into local backup files
    Backup(Backup),
    /// Restore documents from local backup files into a Apache Solr core
    Restore(Restore),
    /// Perform a commit in the Solr core index for persisting documents in disk/memory
    Commit(Execute),
    /// Removes documents from the Solr core definitively
    Delete(Delete),
    /// Generates man page and completion scripts for different shells
    Generate(Generate),
}

#[derive(Parser, Debug)]
pub(crate) struct Backup {
    /// Solr Query param 'q' for filtering which documents are retrieved
    /// See: <https://lucene.apache.org/solr/guide/6_6/the-standard-query-parser.html>
    #[arg(short, long, display_order = 40, value_name = "'f1:vl1 AND f2:vl2'")]
    pub query: Option<String>,

    /// Solr core fields names for sorting documents for retrieval
    #[arg(
        short,
        long,
        display_order = 41,
        value_name = "f1:asc,f2:desc,...",
        value_delimiter = ','
    )]
    pub order: Vec<SortField>,

    /// Skip this quantity of documents in the Solr Query
    #[arg(short = 'k', long, display_order = 42, value_parser = parse_quantity, default_value = "0", value_name = "quantity")]
    pub skip: u64,

    /// Maximum quantity of documents for retrieving from the core (like 100M)
    #[arg(short, long, display_order = 43, value_parser = parse_quantity, value_name = "quantity")]
    pub limit: Option<u64>,

    /// Names of core fields retrieved in each document [default: all but _*]
    #[arg(short, long, display_order = 44, value_name = "field1,field2,...")]
    pub select: Vec<String>,

    /// Slice the queries by using the variables {begin} and {end} for iterating in `--query`
    /// Used in bigger solr cores with huge number of docs because querying the end of docs is expensive and fails frequently
    #[arg(short, long, display_order = 50, default_value_t = IterateMode::Day, value_name = "mode", value_enum)]
    pub iterate_by: IterateMode,

    /// The range of dates/numbers for iterating the queries throught slices.
    /// Requires that the query parameter contains the variables {begin} and {end} for creating the slices.
    /// Use numbers or dates in ISO 8601 format (yyyy-mm-ddTHH:MM:SS)
    #[arg(
        short = 'b',
        long = "between",
        display_order = 51,
        value_name = "begin> <end",
        requires = "query",
        number_of_values = 2
    )]
    pub iterate_between: Vec<String>,

    /// Number to increment each step in iterative mode
    #[arg(
        long = "step",
        display_order = 52,
        default_value_t = 1,
        value_name = "num",
        // value_parser = clap::value_parser!(u64).range(0..366),
    )]
    pub iterate_step: u64,

    /// Number of documents to retrieve from solr in each reader step
    #[arg(long, display_order = 70, default_value = "4k", value_parser = parse_quantity, value_name = "quantity")]
    pub num_docs: u64,

    /// Max number of files of documents stored in each zip file
    #[arg(long, display_order = 71, default_value = "40", value_parser = parse_quantity, value_name = "quantity")]
    pub archive_files: u64,

    /// Optional prefix for naming the zip backup files when storing documents
    #[arg(long, display_order = 72, value_parser = parse_file_prefix, value_name = "name")]
    pub zip_prefix: Option<String>,

    /// Use only when your Solr Cloud returns a distinct count of docs for some queries in a row.
    /// This may be caused by replication problems between cluster nodes of shard replicas of a core.
    /// Response with 'num_found' bellow the greatest value are ignored for getting all possible docs.
    /// Use with `--params shards=shard_name` for retrieving all docs for each shard of the core
    #[arg(
        long,
        display_order = 73,
        default_value = "0",
        value_name = "count",
        // value_parser = clap::value_parser!(u64).range(0..99),
        hide_default_value = true
    )]
    pub workaround_shards: u64,

    #[command(flatten)]
    pub options: CommonArgs,

    #[command(flatten)]
    pub transfer: ParallelArgs,
}

#[derive(Parser, Debug)]
pub(crate) struct Restore {
    /// Mode to perform commits of the documents transaction log while updating the core
    /// [possible values: none, soft, hard, {interval} ]
    #[arg(short, long, display_order = 40, default_value = "hard", value_parser = parse_commit_mode, value_name = "mode")]
    pub flush: CommitMode,

    /// Do not perform a final hard commit before finishing
    #[arg(long, display_order = 41)]
    pub no_final_commit: bool,

    /// Disable core replication at start and enable again at end
    #[arg(long, display_order = 42)]
    pub disable_replication: bool,

    /// Search pattern for matching names of the zip backup files
    #[arg(short, long, display_order = 70, value_name = "core*.zip")]
    pub search: Option<String>,

    /// Optional order for searching the zip archives
    #[arg(
        long,
        display_order = 71,
        default_value = "none",
        hide_possible_values = true,
        hide_default_value = true,
        value_name = "asc | desc"
    )]
    pub order: SortOrder,

    #[command(flatten)]
    pub options: CommonArgs,

    #[command(flatten)]
    pub transfer: ParallelArgs,
}

#[derive(Parser, Debug)]
pub(crate) struct Delete {
    /// Solr Query for filtering which documents are removed in the core.
    /// Use '*:*' for excluding all documents in the core.
    /// There are no way of recovering excluded docs.
    /// Use with caution and check twice.
    #[arg(short, long, display_order = 40, value_name = "f1:val1 AND f2:val2")]
    pub query: String,

    /// Wether to perform a commits of transaction log after removing the documents
    #[arg(short, display_order = 41, long, default_value = "soft", value_name = "mode")]
    pub flush: CommitMode,

    #[command(flatten)]
    pub options: CommonArgs,
}

#[derive(Parser, Debug)]
pub(crate) struct Execute {
    #[command(flatten)]
    pub options: CommonArgs,
}

#[derive(Parser, Debug)]
pub(crate) struct Generate {
    /// Specifies the shell for which the argument completion script should be generated
    #[arg(short, long, display_order = 10, value_parser = parse_shell, required_unless_present_any(["all", "manpage"]))]
    pub shell: Option<Shell>,

    /// Generate a manpage for this application in the output directory
    #[arg(short, long, display_order = 69, requires("output_dir"))]
    pub manpage: bool,

    /// Generate argument completion script in the output directory for all supported shells
    #[arg(short, long, display_order = 70, requires("output_dir"))]
    pub all: bool,

    /// Write the generated assets to <path/to/output/dir> or to stdout if not specified
    #[arg(short, long, display_order = 71, value_name = "path/to/output/dir")]
    pub output_dir: Option<PathBuf>,
}

// #endregion

// #region Cli common

#[derive(Args, Clone, Debug)]
pub(crate) struct CommonArgs {
    /// Url pointing to the Solr cluster
    #[arg(short, long, display_order = 10, env = SOLR_COPY_URL, value_parser = parse_solr_url, value_name = "localhost:8983/solr")]
    pub url: String,

    /// Case sensitive name of the core in the Solr server
    #[arg(short, long, display_order = 20, value_name = "core")]
    pub core: String,

    #[command(flatten)]
    pub logging: LoggingArgs,
}

#[derive(Parser, Clone, Debug)]
pub(crate) struct LoggingArgs {
    /// What level of detail should print messages
    #[arg(long, display_order = 90, value_name = "level", default_value = "info", value_enum)]
    pub log_level: LevelFilter,

    /// Terminal output to print messages
    #[arg(long, display_order = 91, value_name = "mode", default_value = "mixed", value_parser = parse_terminal_mode)]
    pub log_mode: TerminalMode,

    /// Write messages to a local file
    #[arg(long, display_order = 92, value_name = "path")]
    pub log_file_path: Option<PathBuf>,

    /// What level of detail should write messages to the file
    #[arg(
        long,
        display_order = 93,
        value_name = "level",
        default_value = "debug",
        value_enum,
        hide_possible_values = true
    )]
    pub log_file_level: LevelFilter,
}

#[derive(Args, Debug)]
/// Dumps and restores documents from a Apache Solr core into local backup files
pub(crate) struct ParallelArgs {
    /// Existing folder where the zip backup files containing the extracted documents are stored
    #[arg(short, display_order = 30, long, env = SOLR_COPY_DIR, value_name = "/path/to/output")]
    pub dir: PathBuf,

    /// Extra parameter for Solr Update Handler.
    /// See: <https://lucene.apache.org/solr/guide/transforming-and-indexing-custom-json.html>
    #[arg(short, long, display_order = 60, value_name = "useParams=mypars")]
    pub params: Option<String>,

    /// How many times should continue on source document errors
    #[arg(short, long, display_order = 61, default_value = "0", value_name = "count", value_parser = parse_quantity_max)]
    pub max_errors: u64,

    /// Delay before any processing in solr server. Format as: 30s, 15min, 1h
    #[arg(long, display_order = 62, default_value = "0", value_name = "time", value_parser = parse_millis, hide_default_value = true)]
    pub delay_before: u64,

    /// Delay between each http operations in solr server. Format as: 3s, 500ms, 1min
    #[arg(long, display_order = 63, default_value = "0", value_name = "time", value_parser = parse_millis, hide_default_value = true)]
    pub delay_per_request: u64,

    /// Delay after all processing. Usefull for letting Solr breath.
    #[arg(long, display_order = 64, default_value = "0", value_name = "time", value_parser = parse_millis, hide_default_value = true)]
    pub delay_after: u64,

    /// Number parallel threads exchanging documents with the solr core
    #[arg(
        short,
        long,
        display_order = 80,
        default_value = "1",
        value_name = "count",
        // value_parser = clap::value_parser!(u64).range(1..128),
    )]
    pub readers: u64,

    /// Number parallel threads syncing documents with the zip archives
    #[arg(
        short,
        long,
        display_order = 80,
        default_value = "1",
        value_name = "count",
        // value_parser = clap::value_parser!(u64).range(1..=128),
    )]
    pub writers: u64,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Tells Solrt to performs a commit of the updated documents while updating the core
pub(crate) enum CommitMode {
    /// Do not perform commit
    None,
    /// Perform a hard commit by each step for flushing all uncommitted documents in a transaction log to disk
    /// This is the safest and the slowest method
    Hard,
    /// Perform a soft commit of the transaction log for invalidating top-level caches and making documents searchable
    Soft,
    /// Force a hard commit of the transaction log in the defined milliseconds period
    Within { millis: u64 },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
/// Used in bigger solr cores with huge number of docs because querying the end of docs is expensive and fails frequently
pub(crate) enum IterateMode {
    None,
    /// Break the query in slices by a first ordered date field repeating between {begin} and {end} in the query parameters
    Minute,
    Hour,
    Day,
    /// Break the query in slices by a first ordered integer field repeating between {begin} and {end} in the query parameters
    Range,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum SortOrder {
    None,
    Asc,
    Desc,
}

pub(crate) const SOLR_COPY_DIR: &str = "SOLR_COPY_DIR";
pub(crate) const SOLR_COPY_URL: &str = "SOLR_COPY_URL";

// #endregion

// #region Param parsing

fn parse_quantity(src: &str) -> Result<u64, String> {
    lazy_static! {
        static ref REGKB: Regex =
            Regex::new("^([0-9]+)\\s*([k|m|g|t|K|M|G|T](?:[b|B])?)?$").unwrap();
    }
    let up = src.trim().to_ascii_uppercase();

    match REGKB.get_groups(&up) {
        None => Err(format!("Wrong value: '{}'. Use numbers only, or with suffix: K M G", src)),
        Some(parts) => {
            let number = parts.get_as_str(1);
            let multiplier = parts.get_as_str(2);
            let parsed = number.parse::<u64>();
            match parsed {
                Err(_) => Err(format!("Wrong value for number: '{}'", src)),
                Ok(quantity) => match multiplier {
                    "" => Ok(quantity),
                    "K" | "KB" => Ok(quantity * 1000),
                    "M" | "MB" => Ok(quantity * 1_000_000),
                    "G" | "GB" => Ok(quantity * 1_000_000_000),
                    "T" | "TB" => Ok(quantity * 1_000_000_000_000),
                    _ => Err(format!(
                        "Wrong value for quantity multiplier '{}' in '{}'",
                        multiplier, src
                    )),
                },
            }
        }
    }
}

fn parse_quantity_max(s: &str) -> Result<u64, String> {
    let lower = s.to_ascii_lowercase();
    match lower.as_str() {
        "max" => Ok(u64::MAX),
        _ => match parse_quantity(s) {
            Ok(value) => Ok(value),
            Err(_) => Err(format!("'{}'. [alowed: all, <quantity>]", s)),
        },
    }
}

fn parse_millis(src: &str) -> Result<u64, String> {
    lazy_static! {
        static ref REGKB: Regex = Regex::new("^([0-9]+)\\s*([a-zA-Z]*)$").unwrap();
    }
    let lower = src.trim().to_ascii_lowercase();

    match REGKB.get_groups(&lower) {
        None => Err(format!("Wrong interval: '{}'. Use numbers only, or with suffix: s m h", src)),
        Some(parts) => {
            let number = parts.get_as_str(1);
            let multiplier = parts.get_as_str(2);
            let parsed = number.parse::<u64>();
            match parsed {
                Err(_) => Err(format!("Wrong value for number: '{}'", src)),
                Ok(quantity) => match multiplier {
                    "ms" | "millis" | "milliseconds" => Ok(quantity),
                    "" | "s" | "sec" | "secs" | "seconds" => Ok(quantity * 1000),
                    "m" | "min" | "mins" | "minutes" => Ok(quantity * 60_000),
                    "h" | "hr" | "hrs" | "hours" => Ok(quantity * 3_600_000),
                    _ => Err(format!(
                        "Wrong value for time multiplier '{}' in '{}'",
                        multiplier, src
                    )),
                },
            }
        }
    }
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
        return Err("Solr url scheme must be http or https as in: http:://server.domain:8983/solr"
            .to_string());
    }
    if parsed.query().is_some() {
        return Err("Solr url scheme must be a base url without query parameters as in: \
                    http:://server.domain:8983/solr"
            .to_string());
    }
    if parsed.path_segments().is_none() {
        return Err(
            "Solr url path must be 'solr' as in: http:://server.domain:8983/solr".to_string()
        );
    } else {
        let paths = parsed.path_segments();
        if paths.iter().count() != 1 {
            return Err("Solr url path must not include core name as in: \
                        http:://server.domain:8983/solr"
                .to_string());
        }
    }
    Ok(url2)
}

fn parse_file_prefix(src: &str) -> Result<String, String> {
    lazy_static! {
        static ref REGFN: Regex = Regex::new("^(\\w+)$").unwrap();
    }
    match REGFN.get_group(src, 1) {
        None => {
            Err(format!("Wrong output filename: '{}'. Considere using letters and numbers.", src))
        }
        Some(group1) => Ok(group1.to_string()),
    }
}

fn parse_commit_mode(s: &str) -> Result<CommitMode, String> {
    let lower = s.to_ascii_lowercase();
    match lower.as_str() {
        "none" => Ok(CommitMode::None),
        "soft" => Ok(CommitMode::Soft),
        "hard" => Ok(CommitMode::Hard),
        _ => match parse_millis(s) {
            Ok(value) => Ok(CommitMode::Within { millis: value }),
            Err(_) => Err(format!("'{}'. [alowed: none soft hard <secs>]", s)),
        },
    }
}

fn parse_terminal_mode(s: &str) -> Result<TerminalMode, String> {
    let lower = s.to_ascii_lowercase();
    match lower.as_str() {
        "stdout" => Ok(TerminalMode::Stdout),
        "stderr" => Ok(TerminalMode::Stderr),
        "mixed" => Ok(TerminalMode::Mixed),
        _ => Err(format!("Invalid terminal mode: {}. [alowed: stdout stderr mixed]", s)),
    }
}

fn parse_shell(s: &str) -> Result<Shell, String> {
    let full = PathBuf::from(s);
    let invl = format!("Invalid shell: {}", s);
    let name = full.file_name().unwrap().to_str().ok_or(invl.clone())?;
    let lowr = name.to_ascii_lowercase();
    <Shell as FromStr>::from_str(&lowr).map_err(|_| invl)
}

// #endregion

// #region Cli impl

impl Commands {
    pub(crate) fn validate(&self) -> Result<(), String> {
        match self {
            Self::Backup(get) => get.validate(),
            Self::Restore(put) => put.validate(),
            _ => Ok(()),
        }
    }

    pub(crate) fn get_options(&self) -> Option<&CommonArgs> {
        match &self {
            Self::Backup(get) => Some(&get.options),
            Self::Restore(put) => Some(&put.options),
            Self::Commit(com) => Some(&com.options),
            Self::Delete(del) => Some(&del.options),
            _ => None,
        }
    }

    pub(crate) fn get_logging(&self) -> LoggingArgs {
        match self.get_options() {
            None => LoggingArgs::default().clone(),
            Some(opt) => opt.get_logging().clone(),
        }
    }
}

impl CommonArgs {
    pub(crate) fn to_command(&self) -> Execute {
        Execute { options: self.clone() }
    }

    pub(crate) fn get_core_handler_url(&self, handler_url_path: &str) -> String {
        #[rustfmt::skip]
        let parts: Vec<String> = vec![
            self.url.with_suffix("/"),
            self.core.clone(),
            handler_url_path.with_prefix("/"),
        ];
        parts.concat()
    }

    pub(crate) fn get_update_url_with(&self, query_string_params: &str) -> String {
        let parts: Vec<String> =
            vec![self.get_core_handler_url("/update"), query_string_params.with_prefix("?")];
        parts.concat()
    }

    pub(crate) fn get_update_url(&self) -> String {
        self.get_update_url_with(EMPTY_STR)
    }

    pub(crate) fn get_logging(&self) -> &LoggingArgs {
        &self.logging
    }

    pub(crate) fn is_quiet(&self) -> bool {
        self.logging.log_level == LevelFilter::Off
    }
}

impl ParallelArgs {
    pub(crate) fn get_param(&self, separator: &str) -> String {
        self.params.as_ref().unwrap_or(&EMPTY_STRING).with_prefix(separator)
    }
}

impl LoggingArgs {
    pub(crate) fn is_quiet(&self) -> bool {
        self.log_level == LevelFilter::Off
    }
}

impl Default for LoggingArgs {
    fn default() -> Self {
        Self {
            log_level: LevelFilter::Off,
            log_mode: Default::default(),
            log_file_path: Default::default(),
            log_file_level: LevelFilter::Off,
        }
    }
}

impl Default for CommitMode {
    fn default() -> Self {
        CommitMode::Within { millis: 40_000 }
    }
}

impl FromStr for CommitMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_commit_mode(s)
    }
}

impl CommitMode {
    pub(crate) fn as_param(&self, separator: &str) -> String {
        match self {
            CommitMode::Soft => separator.append("softCommit=true"),
            CommitMode::Hard => separator.append("commit=true"),
            CommitMode::Within { millis } => format!("{}commitWithin={}", separator, millis),
            _ => EMPTY_STRING,
        }
    }

    // pub (crate) fn as_xml(&self, separator: &str) -> String {
    //     match self {
    //         CommitMode::Soft => separator.append("<commit />"),
    //         CommitMode::Hard => separator.append("<commit />"),
    //         CommitMode::Within { millis } => {
    //             separator.append(format!("<commitWithin>{}</commitWithin>", millis).as_str())
    //         }
    //         _ => EMPTY_STRING,
    //     }
    // }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum SortDirection {
    Asc,
    Desc,
}

#[derive(Parser, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SortField {
    pub field: String,
    pub direction: SortDirection,
}

impl fmt::Display for SortField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%20{:?}", self.field, self.direction)
    }
}

impl fmt::Debug for SortField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
                    Ok(SortField { field: sort_field, direction: sort_dir })
                }
            }
        }
    }
}

impl Generate {
    pub(crate) fn get_shells(&self) -> Vec<Shell> {
        let sh: Option<Shell> = if self.all { None } else { self.shell };
        match sh {
            Some(sh1) => vec![sh1],
            None => Shell::value_variants().to_vec(),
        }
    }
}

// #endregion

// #region Cli validation

pub(crate) trait Validation {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl Validation for Backup {
    fn validate(&self) -> Result<(), String> {
        assert_dir_exists(&self.transfer.dir)
    }
}

impl Validation for Restore {
    fn validate(&self) -> Result<(), String> {
        assert_dir_exists(&self.transfer.dir)
    }
}

fn assert_dir_exists(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        Err(format!("Missing folder of zip backup files: {:?}", dir))
    } else {
        Ok(())
    }
}

// #endregion

// #endregion

#[cfg(test)]
pub(crate) mod shared {

    use crate::args::{Cli, Commands};
    use clap::Parser;

    pub(crate) const TEST_SELECT_FIELDS: &'static str = "id,date,vehiclePlate";

    impl Cli {
        pub(crate) fn mockup_from(argm: &[&str]) -> Commands {
            Self::parse_from(argm).arguments
        }

        pub(crate) fn mockup_and_panic(argm: &[&str]) -> Commands {
            let unknown = &["--unknown", "argument"];
            let combined = [argm, unknown].concat();
            let res = Self::try_parse_from(combined);
            res.unwrap().arguments
        }

        pub(crate) fn mockup_for_help(argm: &[&str]) {
            match Self::try_parse_from(argm) {
                Ok(ocli) => {
                    panic!("Ok parsing CLI arguments: {} -> {:?}", argm.join(" "), ocli)
                }
                Err(ef) => {
                    println!("Err parsing CLI arguments: {}", ef)
                }
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {

    // #region Mockup

    use super::shared::TEST_SELECT_FIELDS;
    use super::{parse_millis, parse_quantity, Cli, Commands, CommitMode};
    use clap::Parser;
    use clap_complete::Shell::Bash;
    use log::LevelFilter;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    impl Cli {
        pub(crate) fn mockup_args_backup() -> Commands {
            Self::parse_from(TEST_ARGS_BACKUP).arguments
        }

        pub(crate) fn mockup_args_restore() -> Commands {
            Self::parse_from(TEST_ARGS_RESTORE).arguments
        }

        pub(crate) fn mockup_args_commit() -> Commands {
            Self::parse_from(TEST_ARGS_COMMIT).arguments
        }
    }

    // #endregion Mockup

    // #region CLI Args

    const TEST_ARGS_HELP: &'static [&'static str] = &["solrcopy", "--help"];

    const TEST_ARGS_VERSION: &'static [&'static str] = &["solrcopy", "--version"];

    const TEST_ARGS_HELP_BACKUP: &'static [&'static str] = &["solrcopy", "help", "backup"];

    const TEST_ARGS_HELP_RESTORE: &'static [&'static str] = &["solrcopy", "help", "restore"];

    const TEST_ARGS_BACKUP: &'static [&'static str] = &[
        "solrcopy",
        "backup",
        "--url",
        "http://solr-server.com:8983/solr",
        "--core",
        "demo",
        "--dir",
        "./tmp",
        "--query",
        "ownerId:173826 AND date:[{begin} TO {end}]",
        "--order",
        "date:asc,id:desc,vehiclePlate:asc",
        "--select",
        TEST_SELECT_FIELDS,
        "--between",
        "2020-05-01",
        "2020-05-04T11:12:13",
        "--zip-prefix",
        "zip_filename",
        "--skip",
        "3",
        "--limit",
        "42",
        "--num-docs",
        "5",
        "--archive-files",
        "6",
        "--delay-after",
        "5s",
        "--readers",
        "7",
        "--writers",
        "9",
        "--log-level",
        "debug",
        "--log-mode",
        "mixed",
        "--log-file-level",
        "debug",
        "--log-file-path",
        "/tmp/test.log",
    ];

    const TEST_ARGS_RESTORE: &'static [&'static str] = &[
        "solrcopy",
        "restore",
        "--url",
        "http://solr-server.com:8983/solr",
        "--dir",
        "./tmp",
        "--core",
        "target",
        "--search",
        "*.zip",
        "--flush",
        "soft",
        "--delay-per-request",
        "500ms",
        "--log-level",
        "debug",
    ];

    const TEST_ARGS_COMMIT: &'static [&'static str] = &[
        "solrcopy",
        "commit",
        "--url",
        "http://solr-server.com:8983/solr",
        "--core",
        "demo",
        "--log-level",
        "debug",
    ];

    const TEST_ARGS_DELETE: &'static [&'static str] = &[
        "solrcopy",
        "delete",
        "--url",
        "http://solr-server.com:8983/solr",
        "--core",
        "demo",
        "--query",
        "*:*",
        "--flush",
        "hard",
        "--log-level",
        "debug",
        "--log-file-level",
        "error",
    ];

    const TEST_ARGS_GENERATE: &'static [&'static str] =
        &["solrcopy", "generate", "--shell", "bash", "--output-dir", "target"];

    // #endregion

    // #region Tests

    #[test]
    fn check_params_backup() {
        let parsed = Cli::mockup_args_backup();
        match parsed {
            Commands::Backup(get) => {
                assert_eq!(get.options.url, TEST_ARGS_BACKUP[3]);
                assert_eq!(get.options.core, TEST_ARGS_BACKUP[5]);
                assert_eq!(get.transfer.dir.to_str().unwrap(), TEST_ARGS_BACKUP[7]);
                assert_eq!(get.query, Some(TEST_ARGS_BACKUP[9].to_string()));
                assert_eq!(get.skip, 3);
                assert_eq!(get.limit, Some(42));
                assert_eq!(get.num_docs, 5);
                assert_eq!(get.archive_files, 6);
                assert_eq!(get.transfer.readers, 7);
                assert_eq!(get.transfer.writers, 9);
                assert_eq!(get.options.get_logging().log_level, LevelFilter::Debug);
            }
            _ => panic!("command must be 'backup' !"),
        };
    }

    #[test]
    fn check_params_restore() {
        let parsed = Cli::mockup_args_restore();
        match parsed {
            Commands::Restore(put) => {
                assert_eq!(put.options.url, TEST_ARGS_RESTORE[3]);
                assert_eq!(put.transfer.dir.to_str().unwrap(), TEST_ARGS_RESTORE[5]);
                assert_eq!(put.options.core, TEST_ARGS_RESTORE[7]);
                assert_eq!(put.search.unwrap(), TEST_ARGS_RESTORE[9]);
                assert_eq!(put.flush, CommitMode::Soft);
                assert_eq!(put.flush.as_param("?"), "?softCommit=true");
                assert_eq!(put.options.get_logging().log_level, LevelFilter::Debug);
            }
            _ => panic!("command must be 'restore' !"),
        };
    }

    #[test]
    fn check_params_commit() {
        let parsed = Cli::mockup_args_commit();
        match parsed {
            Commands::Commit(put) => {
                assert_eq!(put.options.url, TEST_ARGS_COMMIT[3]);
                assert_eq!(put.options.core, TEST_ARGS_COMMIT[5]);
                assert_eq!(put.options.get_logging().log_level, LevelFilter::Debug);
            }
            _ => panic!("command must be 'commit' !"),
        };
    }

    #[test]
    fn check_params_delete() {
        let parsed = Cli::mockup_from(TEST_ARGS_DELETE);
        match parsed {
            Commands::Delete(res) => {
                assert_eq!(res.options.url, TEST_ARGS_DELETE[3]);
                assert_eq!(res.options.core, TEST_ARGS_DELETE[5]);
                assert_eq!(res.query, TEST_ARGS_DELETE[7]);
                assert_eq!(res.flush, CommitMode::Hard);
                let logs = res.options.get_logging();
                assert_eq!(logs.log_level, LevelFilter::Debug);
                assert_eq!(logs.log_file_level, LevelFilter::Error);
            }
            _ => panic!("command must be 'delete' !"),
        };
    }

    #[test]
    fn check_params_generate() {
        let parsed = Cli::mockup_from(TEST_ARGS_GENERATE);
        match parsed {
            Commands::Generate(res) => {
                assert_eq!(res.shell, Some(Bash));
                assert_eq!(res.output_dir, Some(PathBuf::from("target")));
            }
            _ => panic!("command must be 'generate' !"),
        };
    }

    #[test]
    fn check_params_help() {
        Cli::mockup_for_help(TEST_ARGS_HELP);
    }

    #[test]
    fn check_params_version() {
        Cli::mockup_for_help(TEST_ARGS_VERSION);
    }

    #[test]
    fn check_params_get_help() {
        Cli::mockup_for_help(TEST_ARGS_HELP_BACKUP);
    }

    #[test]
    fn check_params_put_help() {
        Cli::mockup_for_help(TEST_ARGS_HELP_RESTORE);
    }

    #[test]
    #[should_panic]
    fn check_parse_unknown() {
        Cli::mockup_and_panic(&["solrcopy"]);
    }

    #[test]
    #[should_panic]
    fn check_parse_backup_unknown() {
        Cli::mockup_and_panic(TEST_ARGS_BACKUP);
    }

    #[test]
    #[should_panic]
    fn check_parse_restore_unknown() {
        Cli::mockup_and_panic(TEST_ARGS_RESTORE);
    }

    #[test]
    #[should_panic]
    fn check_parse_commit_unknown() {
        Cli::mockup_and_panic(TEST_ARGS_COMMIT);
    }

    #[test]
    #[should_panic]
    fn check_parse_delete_unknown() {
        Cli::mockup_and_panic(TEST_ARGS_DELETE);
    }

    #[test]
    fn check_parse_quantity() {
        assert_eq!(parse_quantity("3k"), Ok(3_000));
        assert_eq!(parse_quantity("4 k"), Ok(4_000));
        assert_eq!(parse_quantity("5kb"), Ok(5_000));
        assert_eq!(parse_quantity("666m"), Ok(666_000_000));
        assert_eq!(parse_quantity("777mb"), Ok(777_000_000));
        assert_eq!(parse_quantity("888mb"), Ok(888_000_000));
        assert_eq!(parse_quantity("999 mb"), Ok(999_000_000));
    }

    #[test]
    fn check_parse_millis() {
        assert_eq!(parse_millis("3ms"), Ok(3));
        assert_eq!(parse_millis("4 ms"), Ok(4));
        assert_eq!(parse_millis("5s"), Ok(5_000));
        assert_eq!(parse_millis("666s"), Ok(666_000));
        assert_eq!(parse_millis("7m"), Ok(420_000));
        assert_eq!(parse_millis("8min"), Ok(480_000));
        assert_eq!(parse_millis("9 minutes"), Ok(540_000));
        assert_eq!(parse_millis("10h"), Ok(36_000_000));
    }

    // #endregion
}

// end of file
