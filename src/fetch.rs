
use regex::Regex;

use super::args::*;
use super::fails::*;
use super::helpers::*;

// region Http Client 

fn http_get_as_text(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::blocking::get(url)?;
    let content = response.text()?;
    Ok(content)
}

// endregion 

// region Solr Core 

impl SolrCore {

    pub fn inspect_core(args: &Arguments) -> Result<SolrCore, BoxedError> {

        let diagnostics_query_url = args.get_query_for_diagnostics();

        let json = http_get_as_text(&diagnostics_query_url)?;
        
        let res = SolrCore::parse_core_schema(&args, &json);
        res
    }

    pub fn get_rows_from<'a>(url: &'a str) -> Result<String, BoxedError> {

        let json = http_get_as_text(url)?;

        let rows = SolrCore::parse_rows_from_query(&json);
        if rows.is_none() {
            throw(format!("Error parsing rows fetched in solr query results: {}", &url))?
        } else {
            let res = rows.unwrap();
            Ok(res)
        }
    }

    fn parse_core_schema(args: &Arguments, json: &str) -> Result<Self, BoxedError> {

        let gets = args.get()?;
        let core_name = &gets.from;

        let total_rows = Self::parse_num_found(json)?;
        if total_rows < 1 { 
            throw(format!("Solr Core '{}'is empty!", core_name))? 
        };

        let parsed_fields = Self::parse_field_names(&json);

        let core_fields = if gets.select.is_empty() {
            match parsed_fields {
                None => throw(format!("Missing fields to parse in Solr Core '{}'!", core_name))?,
                Some(fields) => fields,
            }
        } else {
            // TODO: check if args.select fields matches parsed_fields
            gets.select.clone()
        };
        let res = SolrCore {
            num_found: total_rows,
            fields: core_fields,
        };
        Ok(res)
    }
    
    fn parse_num_found(json: &str) -> Result<u64, BoxedError> {
        lazy_static! {
            static ref REGNF: Regex = Regex::new("\"numFound\":(\\d+),").unwrap();
        }
        match REGNF.get_group(json, 1) {
            None => throw(format!("Error parsing numFound from solr query: {}", json))?,
            Some(group1) => { 
                let res = group1.parse::<u64>();
                res.or_else(|_| { throw::<u64>(format!("Error parsing numFound from solr query: {}", json)) })
            }
        }
    }
        
    fn parse_field_names(json: &str) -> Option<Vec<String>> {
        lazy_static! {
            static ref REGROW: Regex = Regex::new("\\[\\{(.+)\\}\\]").unwrap();
            static ref REGFN: Regex = Regex::new("\"(\\w+)\":").unwrap();
        }
        match REGROW.get_group(json, 1) {
            None => None,
            Some(row1) => {
                let matches = REGFN.get_groups(row1, 1);
                let filtered = matches.iter()
                        .filter(|s| { !s.starts_with("_") })
                        .map(|s| { s.to_string() })
                        .collect::<Vec<String>>();
                Some(filtered)
            }
        }
    }
        
    fn parse_rows_from_query<'a>(json: &'a str) -> Option<String> {
        lazy_static! {
            static ref REGNF: Regex = Regex::new("(\\[.+\\])").unwrap(); // (\[.+\]) or  (\[.+\])(?:\}\})$
        }
        let parsed = REGNF.get_group(json, 1);
        parsed.map(|s| s.to_string())
    }
}

// endregion 

#[cfg(test)]
pub mod test {
    use crate::fetch::*;

    const CORE_1ROW: &str = "{\"response\":{\"numFound\":87,\"start\":0,\"docs\":[{\"date\":\"2018-08-03T00:00:00Z\",\"deviceId\":\"\",\"vehiclePlate\":\"A\\B\",\"ownerId\":\"173826\",\"deviceSerialNumber\":\"205525330\",\"regionType\":\"state\",\"clientId\":[\"2\"],\"proposalNumber\":\"635383717\",\"distanceTraveled\":20030,\"minutesTraveled\":48,\"contracted\":false,\"periodCode\":\"1\",\"groupId\":[\"2\"],\"vehicleNumber\":\"\",\"timeTraveled\":\"00:47:39\",\"identification\":\"Jack Daniesls\",\"policyNumber\":\"172645\",\"id\":\"633040_1533254400_1_5\",\"trackableObjectId\":\"633040\",\"periodType\":\"daily\",\"docNumber\":\"1235232435\",\"regionCode\":\"5\",\"_ttl_\":\"+6MONTHS\",\"_expiration_date_\":\"2019-02-06T16:34:09.326Z\",\"_version_\":1608068103504134147}]}}";
    const CORE_3ROW: &str = "{\"response\":{\"numFound\":17344647,\"start\":0,\"maxScore\":1.0,\"docs\":[{\"date\":\"2019-11-04T00:00:00Z\",\"distanceTraveled\":7462,\"groupId\":[\"390433\"],\"id\":\"633040_1572825600_1_5\",\"regionCode\":\"5\"},{\"date\":\"2019-11-04T00:00:00Z\",\"distanceTraveled\":0,\"groupId\":[\"107509\"],\"id\":\"633133_1572825600_0_0\",\"regionCode\":\"0\"},{\"date\":\"2019-11-04T00:00:00Z\",\"distanceTraveled\":183816,\"groupId\":[\"513858\"],\"id\":\"633238_1572825600_1_4\",\"regionCode\":\"4\"}]}}";

    #[test]
    fn check_schema_num_rows() {

        let num_found = SolrCore::parse_num_found(CORE_1ROW);
        assert_eq!(num_found.ok(), Some(87));
    }

    #[test]
    fn check_schema_fields() {

        let fields = SolrCore::parse_field_names(CORE_1ROW);
        assert_eq!(fields.is_some(), true);

        let fields2 = fields.unwrap();
        // println!(" {:?}", fields2);

        assert_eq!(fields2.len(), 22);
        assert_eq!(fields2.get(0).unwrap(), "date");
        assert_eq!(fields2.get(21).unwrap(), "regionCode");
    }

    #[test]
    fn check_query_rows() {

        let rows = SolrCore::parse_rows_from_query(CORE_3ROW);
        assert_eq!(rows.is_some(), true);

        let rows_text = rows.unwrap();
        assert_eq!(rows_text.starts_with("[{"), true);
        assert_eq!(rows_text.ends_with("}]"), true);
        assert_eq!(rows_text.split("},{").collect::<Vec<&str>>().len(), 3);
    }
}

