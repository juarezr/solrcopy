use crate::args::*;
use crate::helpers::*;

// region Iterator

#[derive(Debug)]
pub struct Steps {
    pub curr: u64,
    pub limit: u64,
    pub batch: u64,
    pub url: String,
}

#[derive(Debug)]
pub struct Step {
    pub curr: u64,
    pub url: String,
}

impl Iterator for Steps {
    type Item = Step;

    fn next(&mut self) -> Option<Step> {
        if self.limit > self.curr {
            let remaining = self.limit - self.curr;
            let rows = self.batch.min(remaining);
            let query = format!("{}&start={}&rows={}", self.url, self.curr, rows);
            let res = Step {
                url: query,
                curr: self.curr,
            };
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

impl Steps {
    pub fn len(&self) -> u64 {
        let res = self.limit / self.batch;
        if self.limit % self.batch == 0 {
            res
        } else {
            res + 1
        }
    }
}

// endregion

// region Solr Core

#[derive(Debug)]
pub struct SolrCore {
    pub num_found: u64,
    pub fields: Vec<String>,
}

impl Backup {
    pub fn get_steps(&self, core_info: &SolrCore) -> Steps {
        let core_fields: &[String] = &core_info.fields;
        let fl = self.get_query_fields(core_fields);
        let query = self.get_query_url(&fl);
        let docs = core_info.num_found.min(self.limit.unwrap_or(std::u64::MAX));
        Steps {
            curr: 0,
            limit: docs,
            batch: self.batch,
            url: query,
        }
    }

    pub fn get_query_fields(&self, core_fields: &[String]) -> String {
        let fields = if self.select.is_empty() {
            core_fields
        } else {
            &self.select
        };
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
        let query = self
            .filter
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
            format!("&q={}", query),
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

    use crate::args::tests::*;
    use crate::args::*;
    use crate::fails::*;
    use crate::helpers::*;
    use crate::steps::SolrCore;

    impl Arguments {
        pub fn get(&self) -> Result<&Backup, BoxedError> {
            match &self {
                Self::Backup(gets) => Ok(&gets),
                _ => raise("command must be 'backup' !"),
            }
        }
    }

    impl SolrCore {
        pub fn mockup() -> Self {
            SolrCore {
                num_found: 100,
                fields: vec![TEST_SELECT_FIELDS.split(COMMA).collect()],
            }
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
        assert_eq!(i, 9);
    }
}
