// region depedencies 

use regex::Regex;

use std::str::FromStr;
use std::fmt;

use structopt::StructOpt;
use url::Url;
use std::path::PathBuf;

use super::helpers::*;
use super::fails::*;

// endregion 

// region Order By 

pub enum SortDirection {
    Asc,
    Desc
}

impl fmt::Debug for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {  SortDirection::Asc => "asc", _ => "desc" })
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
                None    => {
                    Err(s.to_string())
                },
                Some(x) => {
                    let sort_field = x.get(1).unwrap().as_str().to_string();
                    let sort_dir = if x.len() == 4 && x.get(4).unwrap().as_str() == "desc" { 
                        SortDirection::Desc 
                    } else {
                        SortDirection::Asc 
                    };
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

// region Arguments 

#[derive(StructOpt, Debug)]
/// Dumps records from a Apache Solr core into local backup files
pub struct Backup {
    /// Case sensitive name of the Solr core for extracting records
    #[structopt(short, long)]
    pub from: String,

    /// Solr Query filter for filtering returned records  
    #[structopt(short = "w", long = "where")]
    pub filter: Option<String>,

    /// Solr core fields names for restricting columns for retrieval
    #[structopt(short, long)]
    pub select: Vec<String>,

    /// Solr core fields names for sorting records for retrieval (like: field1:desc)
    #[structopt(short, long)]
    pub order: Vec<SortField>,

    /// Maximum number of records for retrieving from the core
    #[structopt(short, long)]
    pub limit: Option<u64>,

    /// Number of records for reading from solr in each step
    #[structopt(short, long, default_value = "4096")]
    pub batch: u64,

     /// Existing folder for writing the dump files
    #[structopt(short, long, parse(from_os_str), env = "SOLROUT_DIR")]
    pub into: PathBuf,

    /// Name for writing backup zip files  
    #[structopt(short, long, parse(try_from_str = parse_file_prefix))]
    pub name: Option<String>,

    #[structopt(flatten)]
    pub options: Options,    
}

#[derive(StructOpt, Debug)]
/// Dumps and restores records from a Apache Solr core into local backup files
pub struct Restore {
    /// Case sensitive name of the Solr core to upload records/data
    #[structopt(short, long)]
    pub into: String,

     /// Existing folder for searching and reading the zip backup files
    #[structopt(short, long, parse(from_os_str), env = "SOLROUT_DIR")]
    pub from: PathBuf,

    /// Pattern for matching backup zip files in `from` folder for restoring
    #[structopt(short, long)]
    pub pattern: Option<String>,

    #[structopt(flatten)]
    pub options: Options,    
}

#[derive(StructOpt, Debug)]
pub enum Arguments {
    /// Dumps records from a Apache Solr core into local backup files
    Backup (Backup),
    /// Restore records from local backup files into a Apache Solr core
    Restore(Restore),
}

#[derive(StructOpt, Debug)]
/// Dumps and restores records from a Apache Solr core into local backup files
pub struct Options {

    /// Url pointing to the Solr base address like: http://solr-server:8983/solr
    #[structopt(short, long, env = "SOLR_URL", parse(try_from_str = parse_solr_url))]
    pub url: String,

    /// Show details of the execution
    #[structopt(long)]
    pub verbose: bool,
}

impl Arguments {

    pub fn parse_from_args() -> Result<Self, BoxedError>  {
        
        let res = Self::from_args();
        let dir = res.dir()?;
        if !dir.exists() {
            throw(format!("Missing folder of zip backup files: {:?}", dir))?
        }
        Ok(res)
    }

    pub fn dir(&self) ->  Result<&PathBuf, BoxedError> {
        let dir = match &self {
            Self::Backup(get) => &get.into,
            Self::Restore(put) => &put.from,
        };
        Ok(dir)
    }
}

fn parse_solr_url(src: &str) -> Result<String, String> {

    let url2 = if src.starts_with_any(&["http://", "https://"]) { src.to_owned() } else { "http://".append(src) };
    let parsing = Url::parse(src);
    if let Err(reason) = parsing {
        return Err(format!("Error parsing Solr: {}", reason));
    }
    let parsed = parsing.unwrap();
    if parsed.scheme() != "http" {
        return Err("Solr url scheme must be http or https as in: http:://server.domain:8983/solr".to_string());
    }
    if parsed.query().is_some() {
        return Err("Solr url scheme must be a base url without query parameters as in: http:://server.domain:8983/solr".to_string());
    }
    if parsed.path_segments().is_none() {
        return Err("Solr url path must be 'solr' as in: http:://server.domain:8983/solr".to_string());
    } else {
        let paths: Vec<&str> = parsed.path_segments().unwrap().collect();
        if paths.len() > 1 {
        return Err("Solr url path must not include core name as in: http:://server.domain:8983/solr".to_string());
        }
    }
    Ok(url2)
}

fn parse_file_prefix(src: &str) -> Result<String, String> {
    lazy_static! {
        static ref REGFN: Regex = Regex::new("^(\\w+)$").unwrap();
    }
    match REGFN.get_group(src, 1) {
        None => Err(format!("Wrong output filename: '{}'. Considere using letters and numbers.", src)),
        Some(group1) => Ok(group1.to_string())
    }
}

// endregion

// endregion

#[cfg(test)]
pub mod tests {

    // region Mockup

    use crate::args::{Arguments};

    use structopt::StructOpt;

    impl Arguments {

        pub fn mockup_from(argument_list: &[&str]) {
            match Self::from_iter_safe(argument_list) {
                Ok(_) => panic!("Error parsing command line arguments: {}", argument_list.join(" ")),
                Err(_) => (),
            }
        }

        pub fn mockup_args_get() -> Self {
            Self::from_iter(TEST_ARGS_GET)
        }

        pub fn mockup_args_put() -> Self {
            Self::from_iter(TEST_ARGS_PUT)
        }
    }
 
    pub const TEST_SELECT_FIELDS: &'static str = "id,date,vehiclePlate";
   
    const TEST_ARGS_HELP: &'static [&'static str] = &["solrcopy", "--help" ];
   
    const TEST_ARGS_VERSION: &'static [&'static str] = &["solrcopy", "--version" ];
   
    const TEST_ARGS_GET_HELP: &'static [&'static str] = &["solrcopy", "help", "backup" ];
   
    const TEST_ARGS_PUT_HELP: &'static [&'static str] = &["solrcopy", "help", "restore" ];

    const TEST_ARGS_GET: &'static [&'static str] = &[
            "solrcopy",
            "backup", 
            "--url", "http://solr-server.com:8983/solr", 
            "--from", "mileage", 
            "--into", "./tmp",
            "--where", "ownerId:173826 AND periodCode:1", 
            "--order", "date:asc", "id:desc", "vehiclePlate:asc",
            "--select", TEST_SELECT_FIELDS, 
            "--name", "output_filename",
            "--limit", "42", 
            "--batch", "5", 
            "--verbose", 
        ];

    const TEST_ARGS_PUT: &'static [&'static str] = &[
        "solrcopy", 
        "restore", 
        "--url", "http://solr-server.com:8983/solr", 
        "--from", "./tmp",
        "--into", "mileage",
        "--pattern", "*.zip",
        "--verbose", 
    ];

    // endregion

    #[test]
    fn check_params_backup() {

        let parsed = Arguments::mockup_args_get();
        match parsed {
            Arguments::Backup(get) => {
                assert_eq!(get.options.url, TEST_ARGS_GET[3]);
                assert_eq!(get.from, TEST_ARGS_GET[5]);
                assert_eq!(get.into.to_str().unwrap(), TEST_ARGS_GET[7]);
                assert_eq!(get.filter, Some(TEST_ARGS_GET[9].to_string()));
                assert_eq!(get.limit, Some(42));
                assert_eq!(get.batch, 5);
                assert_eq!(get.options.verbose, true);
                },
            _ => panic!("command must be 'backup' !"),
        };
    }

    #[test]
    fn check_params_restore() {

        let parsed = Arguments::mockup_args_put();
        match parsed {
            Arguments::Restore(put) => {
                assert_eq!(put.options.url, TEST_ARGS_PUT[3]);
                assert_eq!(put.from.to_str().unwrap(), TEST_ARGS_PUT[5]);
                assert_eq!(put.into, TEST_ARGS_PUT[7]);
                assert_eq!(put.pattern.unwrap(), TEST_ARGS_PUT[9]);
                assert_eq!(put.options.verbose, true);
                },
            _ => panic!("command must be 'restore' !"),
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
        Arguments::mockup_from(TEST_ARGS_GET_HELP);
    }

    #[test]
    fn check_params_put_help() {
        Arguments::mockup_from(TEST_ARGS_PUT_HELP);
    }
}

// end of file
