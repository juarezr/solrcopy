use super::{args::Backup, connection::SolrClient, fails::*, helpers::*, steps::SolrCore};
use log::debug;
use regex::Regex;

// region Solr Core

impl Backup {
    pub(crate) fn inspect_core(&self) -> BoxedResult<SolrCore> {
        let diagnostics_query_url = self.get_query_for_diagnostics();
        debug!("Inspecting schema of core {} at: {}", self.options.core, diagnostics_query_url);

        // try sometimes for finding the greatest num_found of docs answered by the core
        // Used for fixing problems with corrupted replicas of cores with more than 1 shard
        let times = (self.workaround_shards * 5) + 1;

        let mut res = SolrCore { num_found: 0, fields: vec![] };
        for it in 0..times {
            let json = SolrClient::query_get_as_text(&diagnostics_query_url)?;
            if let Ok(next) = SolrCore::parse_core_schema(self, &json) {
                debug!("#{} Solr query returned num_found: {}", it, next.num_found);
                if next.num_found > res.num_found {
                    res = next;
                }
            }
        }
        if res.num_found <= self.skip {
            throw(format!(
                "Requested {} in --skip but found {} docs with the query.",
                self.skip, res.num_found
            ))?;
        }
        debug!("Core schema: {:?}", res);
        Ok(res)
    }
}

impl SolrCore {
    fn parse_core_schema(gets: &Backup, json: &str) -> BoxedResult<Self> {
        let core_name = &gets.options.core;

        let total_docs = Self::parse_num_found(json)?;
        if total_docs < 1 {
            throw(format!("Solr Core '{}'is empty!", core_name))?
        };
        let parsed_fields = Self::parse_field_names(json);

        let core_fields = if gets.select.is_empty() {
            match parsed_fields {
                None => throw(format!("Missing fields to parse in Solr Core '{}'!", core_name))?,
                Some(fields) => fields,
            }
        } else {
            // TODO: check if args.select fields matches parsed_fields when --validate
            gets.select.clone()
        };
        let res = SolrCore { num_found: total_docs, fields: core_fields };
        Ok(res)
    }

    pub(crate) fn parse_num_found(json: &str) -> BoxedResult<u64> {
        lazy_static! {
            static ref REGNF: Regex = Regex::new("\"numFound\":(\\d+),").unwrap();
        }
        match REGNF.get_group(json, 1) {
            None => throw(format!("Error parsing numFound from solr query: {}", json))?,
            Some(group1) => {
                let res = group1.parse::<u64>();
                res.or_else(|_| {
                    throw::<u64>(format!("Error converting numFound from solr query: {}", json))
                })
            }
        }
    }

    fn parse_field_names(json: &str) -> Option<Vec<String>> {
        lazy_static! {
            static ref REGFN: Regex = Regex::new("\"(\\w+)\":").unwrap();
        }
        let row1 = Self::parse_docs_from_query(json)?;

        let matches = REGFN.get_group_values(row1, 1);
        let filtered = matches
            .iter()
            .filter(|s| !s.starts_with('_'))
            .map(|&s| s.to_string())
            .collect::<Vec<String>>();
        Some(filtered)
    }

    /// Strips out: `[{  "a": "b", "c": "d" }]` from Solr json response
    /// ``` json
    /// {"response":{"numFound":46,"start":0,"docs":_____}}
    /// ```
    pub(crate) fn parse_docs_from_query(json: &str) -> Option<&str> {
        json.find_text_between("docs\":", "}}") // -> [{  ... }]
    }
}

// endregion

#[cfg(test)]
mod tests {
    use super::{SolrCore, StringHelpers};
    use pretty_assertions::assert_eq;

    const CORE_1ROW: &str = r#"{
        "response":{"numFound":46,"start":0,
            "docs":[
                {
                "id":"3007WFP",
                "name":["Dell Widescreen UltraSharp 3007WFP"],
                "cat":["electronics and computer1"],
                "price":[2199.0]}
            ]}}"#;
    const CORE_3ROW: &str = r#"{"response":{"numFound":46,"start":0,
            "docs":[
                {"id":"3007WFP","name":["Dell Widescreen UltraSharp 3007WFP"],"cat":["electronics and computer1"],"price":[2199.0]},
                {"id":"100-435805","name":["ATI Radeon X1900 XTX 512 MB PCIE Video Card"],"cat":["electronics","graphics card"],"price":[649.99]},
                {"id":"EN7800GTX/2DHTV/256M","name":["ASUS Extreme N7800GTX/2DHTV (256 MB)"],"cat":["electronics","graphics card"],"price":[479.95]}
            ]}}"#;

    #[test]
    fn check_schema_num_found() {
        let num_found = SolrCore::parse_num_found(CORE_1ROW);
        assert_eq!(num_found.ok(), Some(46));
    }

    #[test]
    fn check_schema_fields() {
        let fields = SolrCore::parse_field_names(CORE_1ROW);
        assert_eq!(fields.is_some(), true);

        let fields2 = fields.unwrap();

        assert_eq!(fields2.len(), 4);
        assert_eq!(fields2.get(0).unwrap(), "id");
        assert_eq!(fields2.get(1).unwrap(), "name");
        assert_eq!(fields2.get(2).unwrap(), "cat");
        assert_eq!(fields2.get(3).unwrap(), "price");
    }

    #[test]
    fn check_query_docs() {
        let docs = SolrCore::parse_docs_from_query(CORE_3ROW);
        assert_eq!(docs.is_some(), true);

        let json = docs.unwrap().remove_whitespace();

        let starting = &json[..2];
        assert_eq!(starting, "[{");

        let two = json.len() - 2;
        let ending = &json[two..];
        assert_eq!(ending, "}]");

        let rows = json.split("},{").collect::<Vec<&str>>();
        assert_eq!(rows.len(), 3);
    }
}
