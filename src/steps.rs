use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};

use crate::{
    args::{Backup, IterateMode},
    fails::*,
    helpers::*,
};

// region Struct

#[derive(Debug)]
pub struct Slices<T> {
    pub curr: T,
    pub end: T,
    pub increment: usize,
    pub mode: IterateMode,
}

#[derive(Debug)]
pub struct Range {
    pub begin: String,
    pub end: String,
}

#[derive(Debug)]
pub struct Requests {
    pub curr: usize,
    pub limit: usize,
    pub num_docs: usize,
    pub url: String,
}

#[derive(Debug)]
pub struct Step {
    pub curr: usize,
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

impl<T> Slices<T> {
    pub fn from_steps(num_steps: usize, increment_size: usize) -> BoxedResult<Slices<usize>> {
        Ok(Slices::<usize> {
            curr: 0,
            mode: IterateMode::None,
            increment: increment_size,
            end: num_steps,
        })
    }

    pub fn from_range(begin: &str, end: &str, step_size: usize) -> BoxedResult<Slices<usize>> {
        let v1 = Self::parse_between_number(begin)?;
        let v2 = Self::parse_between_number(end)?;
        Ok(Slices::<usize> { curr: v1, end: v2, increment: step_size, mode: IterateMode::Range })
    }

    pub fn from_dates(
        date_mode: IterateMode, begin: &str, end: &str, step_size: usize,
    ) -> BoxedResult<Slices<NaiveDateTime>> {
        let v1 = Self::parse_between_date(begin)?;
        let v2 = Self::parse_between_date(end)?;
        Ok(Slices::<NaiveDateTime> { curr: v1, end: v2, increment: step_size, mode: date_mode })
    }

    fn parse_between_number(value: &str) -> BoxedResult<usize> {
        let parsed = value.parse::<usize>();
        match parsed {
            Err(_) => throw(format!("Wrong value for number: '{}'", value)),
            Ok(quantity) => Ok(quantity),
        }
    }

    fn parse_between_date(value: &str) -> BoxedResult<NaiveDateTime> {
        if value.contains('T') {
            let time = value.parse::<NaiveDateTime>();
            match time {
                Err(_) => throw(format!("Wrong value for date: '{}'", value)),
                Ok(quantity) => Ok(quantity),
            }
        } else {
            let date = value.parse::<NaiveDate>();
            match date {
                Err(_) => throw(format!("Wrong value for date: '{}'", value)),
                Ok(quantity) => Ok(quantity.and_hms(0, 0, 0)),
            }
        }
    }

    fn get_interval(&self, less: i64) -> Duration {
        let plus = self.increment.to_i64();
        let dur = match self.mode {
            IterateMode::Minute => Duration::minutes(plus),
            IterateMode::Hour => Duration::hours(plus),
            IterateMode::Day => Duration::days(plus),
            _ => Duration::days(365),
        };
        dur - Duration::seconds(less)
    }
}

impl Iterator for Slices<usize> {
    type Item = Range;

    fn next(&mut self) -> Option<Range> {
        if self.end > self.curr {
            let next = self.curr + self.increment;
            let last = next - 1;
            let res = Range { begin: self.curr.to_string(), end: last.to_string() };
            self.curr = next;
            Some(res)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_steps = self.end - self.curr + 1;
        if num_steps == 0 {
            (0, None)
        } else {
            (0, Some(num_steps))
        }
    }
}

impl Iterator for Slices<NaiveDateTime> {
    type Item = Range;

    fn next(&mut self) -> Option<Range> {
        if self.end > self.curr {
            let last = self.curr + self.get_interval(1);
            let last2 = if last < self.end { last } else { self.end };
            let res = Range { begin: self.curr.to_string() + "Z", end: last2.to_string() + "Z" };
            let next = self.get_interval(0);
            self.curr += next;
            Some(res)
        } else {
            None
        }
    }
}

impl Range {
    pub fn fill_range(&self, url: &str) -> String {
        let start = url.replace("{begin}", self.begin.as_str());
        start.replace("{end}", self.end.as_str())
    }
}

impl Requests {
    pub fn len(&self) -> usize {
        let res = self.limit / self.num_docs;
        if self.limit % self.num_docs == 0 {
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
            let rows = self.num_docs.min(remaining);
            let query = format!("{}&start={}&rows={}", self.url, self.curr, rows);
            let res = Step { url: query, curr: self.curr };
            self.curr += self.num_docs;
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
        let prefix = match &self.zip_prefix {
            Some(text) => text.to_string(),
            None => {
                let now: DateTime<Utc> = Utc::now();
                let time = now.format("%Y%m%d_%H%M");
                format!("{}_at_{}", &self.options.core, &time)
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
        Requests { curr: self.skip, limit: end_limit, num_docs: self.num_docs, url: query }
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
            self.options.core.clone(),
            "/select?wt=json&indent=off&omitHeader=true".to_string(),
            format!("&q={}", filter),
            sort,
            self.transfer.get_param("&"),
            selected.to_string(),
        ];
        parts.concat()
    }
}

// endregion

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    // region mockup

    use crate::{
        args::{tests::*, *},
        fails::*,
        helpers::*,
        steps::{Slices, SolrCore},
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

    // region iterators

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

    #[test]
    fn check_iterator_for_slices_usize() {
        let slices = Slices::<usize>::from_steps(16, 2);
        assert!(slices.is_ok(), true);

        if let Ok(seq) = slices {
            for step in seq {
                // print!("# {} -> {}", step.begin, step.end);
                assert_eq!(step.begin < step.end, true)
            }
        }
    }

    #[test]
    fn check_iterator_for_slices_datetime() {
        let slices = Slices::<NaiveDateTime>::from_dates(
            IterateMode::Day,
            "2020-04-01",
            "2020-04-03T11:12:13",
            1,
        );
        assert!(slices.is_ok(), true);

        if let Ok(seq) = slices {
            for step in seq {
                // print!("# {} -> {}", step.begin, step.end);
                assert_eq!(step.begin < step.end, true)
            }
        }
    }

    // endregion
}
