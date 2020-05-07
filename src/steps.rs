use chrono::{DateTime, Utc};

use crate::{args::Backup, helpers::*};

// region Struct

#[derive(Debug)]
pub struct Step {
    pub curr: usize,
    pub url: String,
}

#[derive(Debug)]
pub struct Requests {
    pub curr: usize,
    pub limit: usize,
    pub batch: usize,
    pub url: String,
}

#[derive(Debug)]
pub struct Documents {
    pub step: Step,
    pub docs: String,
}

#[derive(Debug)]
pub struct SolrCore {
    pub num_found: usize,
    pub fields: Vec<String>,
}

// endregion

// region Iterators

impl Requests {
    pub fn len(&self) -> usize {
        let res = self.limit / self.batch;
        if self.limit % self.batch == 0 {
            res
        } else {
            res + 1
        }
    }
}

impl Iterator for Requests {
    type Item = Step;

    fn next(&mut self) -> Option<Step> {
        if self.limit > self.curr {
            let remaining = self.limit - self.curr;
            let rows = self.batch.min(remaining);
            let query = format!("{}&start={}&rows={}", self.url, self.curr, rows);
            let res = Step { url: query, curr: self.curr };
            self.curr += self.batch;
            Some(res)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_steps = self.len();
        if num_steps == 0 {
            (0, None)
        } else {
            let max: usize = num_steps.to_usize();
            (0, Some(max))
        }
    }
}

// endregion

// region Solr requests

impl Backup {
    pub fn get_archive_pattern(&self, num_found: usize) -> String {
        let prefix = match &self.prefix {
            Some(text) => text.to_string(),
            None => {
                let now: DateTime<Utc> = Utc::now();
                let time = now.format("%Y%m%d_%H%M");
                format!("{}_at_{}", &self.from, &time)
            }
        };
        format!("{}_docs_{}_seq_{}.zip", prefix, num_found, BRACKETS)
    }

    pub fn get_docs_to_retrieve(&self, core_info: &SolrCore) -> usize {
        core_info.num_found.min(self.limit.unwrap_or(std::usize::MAX))
    }

    pub fn get_steps(&self, core_info: &SolrCore) -> Requests {
        let core_fields: &[String] = &core_info.fields;
        let fl = self.get_query_fields(core_fields);
        let query = self.get_query_url(&fl);
        let end_limit = self.get_docs_to_retrieve(core_info);
        Requests { curr: self.skip, limit: end_limit, batch: self.docs_per_step, url: query }
    }

    pub fn get_query_fields(&self, core_fields: &[String]) -> String {
        let fields = if self.select.is_empty() { core_fields } else { &self.select };
        if fields.is_empty() {
            EMPTY_STRING
        } else {
            let all = fields.join(COMMA);
            "&fl=".append(&all)
        }
    }

    pub fn get_query_for_diagnostics(&self) -> String {
        let url = self.get_query_url(EMPTY_STR);
        format!("{}&start=0&rows=1", url)
    }

    pub fn get_query_url(&self, selected: &str) -> String {
        let filter = self
            .query
            .as_deref()
            .unwrap_or("*:*")
            .replace(" or ", " OR ")
            .replace(" and ", " AND ")
            .replace(" not ", " NOT ")
            .replace(" ", "%20");

        let sort: String = if self.order.is_empty() {
            EMPTY_STRING
        } else {
            let all: Vec<String> = self.order.iter().map(|field| field.to_string()).collect();
            let joined = all.join(COMMA);
            "&sort=".append(&joined)
        };
        let parts = vec![
            self.options.url.with_suffix("/"),
            self.from.clone(),
            "/select?wt=json&indent=off&omitHeader=true".to_string(),
            format!("&q={}", filter),
            sort,
            selected.to_string(),
        ];
        parts.concat()
    }
}

// endregion

#[cfg(test)]
mod tests {
    // region mockup

    use crate::{
        args::{tests::*, *},
        fails::*,
        helpers::*,
        steps::SolrCore,
    };

    impl Arguments {
        pub fn get(&self) -> BoxedResult<&Backup> {
            match &self {
                Self::Backup(gets) => Ok(&gets),
                _ => raise("command must be 'backup' !"),
            }
        }
    }

    impl SolrCore {
        pub fn mockup() -> Self {
            SolrCore { num_found: 100, fields: vec![TEST_SELECT_FIELDS.split(COMMA).collect()] }
        }
    }

    // endregion

    #[test]
    fn check_iterator_for_params_get() {
        let parsed = Arguments::mockup_args_backup();
        let gets = parsed.get().unwrap();
        let core_info = SolrCore::mockup();
        let query = gets.get_query_url(EMPTY_STR);

        let mut i = 0;
        for step in gets.get_steps(&core_info) {
            let url = step.url;
            assert_eq!(url.is_empty(), false);
            assert_eq!(url.starts_with(&query), true);
            i += 1;
        }
        assert_eq!(i, 8);
    }
}
