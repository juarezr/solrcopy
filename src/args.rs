// region depedencies 

use regex::Regex;

use std::str::FromStr;
use std::fmt;

use structopt::StructOpt;
use url::Url;

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
                    let field = x.get(1).unwrap().as_str().to_string();
                    let ord = if x.len() == 4 && x.get(4).unwrap().as_str() == "desc" { 
                        SortDirection::Desc 
                    } else {
                        SortDirection::Asc 
                    };
                    Ok(SortField {
                        field: field,
                        direction: ord,
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
/// 
/// The backup files can be futher reimported using solringest
pub struct Arguments {
    /// Url pointing to the Solr base address like: http://solr-server:8983/solr
    #[structopt(short, long, env = "SOLR_URL", parse(try_from_str = parse_solr_url))]
    pub from: String,

    /// Case sensitive name of the Solr core to extract data
    #[structopt(short, long)]
    pub core: String,

    /// Query filter for Solr filter returned records  
    #[structopt(short = "w", long = "where")]
    pub filter: Option<String>,

    /// Solr core fields names for restricting columns to fetch
    #[structopt(short, long)]
    pub select: Vec<String>,

    /// Solr core fields names for sorting result like: field1:desc
    #[structopt(short, long)]
    pub order: Vec<SortField>,

    /// Maximum number of records for dumping from the core
    #[structopt(short, long)]
    pub limit: Option<u64>,

    /// Number of records for reading from solr in each step
    #[structopt(short, long, default_value = "4096")]
    pub batch: u64,

     /// Existing folder for writing the dump files
    #[structopt(short, long, parse(from_os_str), env = "SOLRDUMP_DIR")]
    pub into: std::path::PathBuf,

    /// Show details of the execution
    #[structopt(short, long)]
    pub verbose: bool,
}

impl Arguments {

    pub fn parse_from_args() -> Result<Self, BoxedError>  {
        
        let res = Self::from_args();
        if !res.into.exists() {
            throw(format!("Wrong folder for writing results: {:?}", &res.into))?
        }
        Ok(res)
    }
}

fn parse_solr_url(src: &str) -> Result<String, String> {

    let url2 = if src.starts_with_any(&["http://", "https://"]) { src.to_owned() } else { "http://".append(src) };
    let parsing = Url::parse(src);
    if let Err(reason) = parsing {
        return Err(format!("Error parsing Solr: {}", reason).to_string());
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

// endregion

// region Solr Core

#[derive(Debug)]
pub struct SolrCore {
    pub num_found: u64,
    pub fields: Vec<String>,
}

// endregion

#[cfg(test)]
pub mod test {
    use crate::args::Arguments;
    use crate::args::SolrCore;
    use crate::helpers::*;

    use structopt::StructOpt;

    impl Arguments {

        pub fn parse_from(argument_list: &[&str]) -> Self {
            Self::from_iter(argument_list)
        }

        pub fn mockup_args1() -> Self {
            Arguments::from_iter(TEST_ARGS1)
        }
    }

    impl SolrCore {

        pub fn mockup() -> Self {
            SolrCore {
                num_found: 100,
                fields: vec!(TEST_SELECT_FIELDS.split(COMMA).collect())
            }
        }
    }
 
    const TEST_SELECT_FIELDS: &'static str = "id,date,vehiclePlate";
   
    const TEST_ARGS0: &'static [&'static str] = &[
            "solrdump", 
            "--help", 
        ];

    const TEST_ARGS1: &'static [&'static str] = &[
            "solrdump", 
            "--from", "http://solr-telematics.ceabsservicos.com.br:8983/solr", 
            "--core", "mileage", 
            "--where", "ownerId:173826 AND periodCode:1", 
            "--order", "date:asc", "id:desc", "vehiclePlate:asc",
            "--select", TEST_SELECT_FIELDS, 
            "--into", "./test",
            "--limit", "42", 
            "--batch", "5", 
            "--verbose", 
        ];

    #[test]
    fn check_params_validity() {

        let parsed = Arguments::mockup_args1();

        assert_eq!(parsed.from, TEST_ARGS1[2]);
        assert_eq!(parsed.core, TEST_ARGS1[4]);
        assert_eq!(parsed.filter, Some(TEST_ARGS1[6].to_string()));
        assert_eq!(parsed.limit, Some(42));
        assert_eq!(parsed.batch, 5);
    }

    #[test]
    fn check_params_help() {
        Arguments::parse_from(TEST_ARGS0);
    }
}

// end of file
