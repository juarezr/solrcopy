use super::{
    args::{Backup, IterateMode},
    fails::{BoxedResult, throw},
    helpers::{BRACKETS, COMMA, EMPTY_STR, EMPTY_STRING},
    helpers::{IntegerHelpers, StringHelpers, replace_solr_date, solr_query},
};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};

// region Struct

#[derive(Debug)]
pub(crate) struct Slices<T> {
    pub curr: T,
    pub end: T,
    pub increment: u64,
    pub mode: IterateMode,
}

#[derive(Debug)]
pub(crate) struct SliceItem {
    pub begin: String,
    pub end: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Requests {
    pub curr: u64,
    pub limit: u64,
    pub num_docs: u64,
    pub url: String,
}

#[derive(Debug)]
pub(crate) struct Step {
    pub curr: u64,
    pub url: String,
}

#[derive(Debug)]
pub(crate) struct Documents {
    pub step: Step,
    pub docs: String,
}

#[derive(Debug)]
pub(crate) struct SolrCore {
    pub num_found: u64,
    pub fields: Vec<String>,
}

// endregion

// region Iterators

impl Slices<String> {
    pub(crate) fn get_iterator(&self) -> Box<dyn Iterator<Item = SliceItem>> {
        if self.curr.is_empty() {
            return Box::new(Self::get_slice_of(1, 1));
        }
        let res: Box<dyn Iterator<Item = SliceItem>> = match self.mode {
            IterateMode::None => Box::new(Self::get_slice_of(1, 1)),
            IterateMode::Range => Box::new(self.get_range_slices().unwrap()),
            _ => Box::new(self.get_period_slices().unwrap()),
        };
        res
    }

    pub(crate) fn estimate_steps(&self) -> BoxedResult<u64> {
        if self.curr.is_empty() {
            return Ok(1);
        }
        let num: u64 = match self.mode {
            IterateMode::None => 1,
            IterateMode::Range => self.get_range_slices()?.len(),
            _ => self.get_period_slices()?.len(),
        };
        let rem = num % self.increment;
        if rem > 0 { Ok(num + 1) } else { Ok(num) }
    }

    fn get_slice_of(num: u64, incr: u64) -> Slices<u64> {
        Slices::<u64> { curr: 0, end: num, mode: IterateMode::Range, increment: incr }
    }

    fn get_range_slices(&self) -> BoxedResult<Slices<u64>> {
        let v1 = Self::parse_between_number(self.curr.as_str())?;
        let v2 = Self::parse_between_number(self.end.as_str())?;
        Ok(Slices::<u64> { curr: v1, end: v2, increment: self.increment, mode: IterateMode::Range })
    }

    fn get_period_slices(&self) -> BoxedResult<Slices<NaiveDateTime>> {
        let v1 = Self::parse_between_date(self.curr.as_str())?;
        let v2 = Self::parse_between_date(self.end.as_str())?;
        Ok(Slices::<NaiveDateTime> {
            curr: v1,
            end: v2,
            increment: self.increment,
            mode: self.mode,
        })
    }

    fn parse_between_number(value: &str) -> BoxedResult<u64> {
        let parsed = value.parse::<u64>();
        match parsed {
            Err(_) => throws!("Wrong value for number: {}", value),
            Ok(quantity) => Ok(quantity),
        }
    }

    fn parse_between_date(value: &str) -> BoxedResult<NaiveDateTime> {
        if value.contains('T') {
            let time = value.parse::<NaiveDateTime>();
            match time {
                Err(_) => throws!("Wrong value for datetime: {}", value),
                Ok(quantity) => Ok(quantity),
            }
        } else {
            let date = value.parse::<NaiveDate>();
            match date {
                Err(_) => throw(format!("Wrong value for date: '{}'", value)),
                Ok(quantity) => quantity
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(should_fail!("Wrong value for date: '{}'", value)),
            }
        }
    }
}

impl Slices<NaiveDateTime> {
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

    fn len(&self) -> u64 {
        let dur = self.end - self.curr;
        let (diff, prev, div) = match self.mode {
            IterateMode::Minute => (dur.num_minutes(), dur.num_seconds(), 60i64),
            IterateMode::Hour => (dur.num_hours(), dur.num_minutes(), 60i64),
            IterateMode::Day => (dur.num_days(), dur.num_hours(), 24i64),
            _ => (1i64, 1i64, 1i64),
        };
        if diff < 0 {
            0
        } else {
            let rem = prev % div;
            let res = if rem == 0 { diff } else { diff + 1 };
            res.to_u64()
        }
    }
}

impl Slices<u64> {
    fn len(&self) -> u64 {
        self.end - self.curr + 1
    }
}

impl Iterator for Slices<u64> {
    type Item = SliceItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end > self.curr {
            let next = self.curr + self.increment;
            let last = next - 1;
            let res = SliceItem { begin: self.curr.to_string(), end: last.to_string() };
            self.curr = next;
            Some(res)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_steps = self.len();
        if num_steps == 0 { (0, None) } else { (0, Some(num_steps.to_usize())) }
    }
}

impl Iterator for Slices<NaiveDateTime> {
    type Item = SliceItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end > self.curr {
            let last = self.curr + self.get_interval(1);
            let part = if last < self.end { last } else { self.end };
            let res = SliceItem { begin: format_solr_time(self.curr), end: format_solr_time(part) };
            let next = self.get_interval(0);
            self.curr += next;
            Some(res)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_steps = self.len();
        if num_steps == 0 { (0, None) } else { (0, Some(num_steps.to_usize())) }
    }
}

impl SliceItem {
    pub(crate) fn filter(&self, step: Step) -> Step {
        if self.begin.is_empty() {
            step
        } else {
            let query =
                replace_solr_vars(step.url.as_str(), self.begin.as_str(), self.end.as_str());
            Step { url: query, curr: step.curr }
        }
    }
}

impl Requests {
    pub(crate) fn len(&self) -> u64 {
        let res = self.limit / self.num_docs;
        if self.limit % self.num_docs == 0 { res } else { res + 1 }
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

fn replace_solr_vars(query: &str, begin: &str, end: &str) -> String {
    let query2 = replace_solr_date(query, "{begin}", begin);
    replace_solr_date(&query2, "{end}", end)
}

fn format_solr_time(date_time: NaiveDateTime) -> String {
    date_time.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

// endregion

// region Solr requests

impl Backup {
    pub(crate) fn get_archive_pattern(&self, num_found: u64) -> String {
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

    pub(crate) fn estimate_docs_quantity(
        &self, schema: &SolrCore, slices: &Slices<String>,
    ) -> BoxedResult<u64> {
        let end_limit = self.get_docs_to_retrieve(schema);
        let num_retrieving = end_limit - self.skip;

        let slice_count = slices.estimate_steps()?;
        Ok(num_retrieving * slice_count)
    }

    pub(crate) fn get_docs_to_retrieve(&self, schema: &SolrCore) -> u64 {
        schema.num_found.min(self.limit.unwrap_or(u64::MAX))
    }

    pub(crate) fn get_steps(&self, schema: &SolrCore) -> Requests {
        let core_fields: &[String] = &schema.fields;
        let fl = self.get_query_fields(core_fields);
        let query = self.get_query_url(&fl, true);
        let end_limit = self.get_docs_to_retrieve(schema);
        Requests { curr: self.skip, limit: end_limit, num_docs: self.num_docs, url: query }
    }

    pub(crate) fn get_query_fields(&self, core_fields: &[String]) -> String {
        let fields = if self.select.is_empty() { core_fields } else { &self.select };
        if fields.is_empty() {
            EMPTY_STRING
        } else {
            let all = fields.join(COMMA);
            "&fl=".append(&all)
        }
    }

    pub(crate) fn get_query_for_diagnostics(&self) -> String {
        let url = self.get_query_url(EMPTY_STR, false);
        format!("{}&start=0&rows=1", url)
    }

    pub(crate) fn replace_vars(&self, query: &str, raw: bool) -> String {
        if raw || self.iterate_between.is_empty() {
            query.to_string()
        } else {
            let (begin, end) = self.get_between();
            replace_solr_vars(query, begin, end)
        }
    }

    pub(crate) fn get_query_url(&self, selected: &str, raw: bool) -> String {
        let qparam = self.query.as_deref().unwrap_or("*:*");
        let qfixed = self.replace_vars(qparam, raw);
        let filterq = solr_query(&qfixed);
        let fqparam = self.fq.as_deref().unwrap_or("*:*");
        let filterfq = solr_query(fqparam);

        let sort: String = if self.order.is_empty() {
            EMPTY_STRING
        } else {
            let all: Vec<String> = self.order.iter().map(|field| field.to_string()).collect();
            let joined = all.join(COMMA);
            "&sort=".append(&joined)
        };
        let parts = [
            self.options.url.with_suffix("/"),
            self.options.core.clone(),
            "/select?wt=json&indent=off&omitHeader=true".to_string(),
            format!("&q={}", filterq),
            format!("&fq={}", filterfq),
            sort,
            self.transfer.get_param("&"),
            selected.to_string(),
        ];
        parts.concat()
    }

    pub(crate) fn get_slices(&self) -> Slices<String> {
        let (begin, end) = self.get_between();
        Slices::<String> {
            curr: begin.to_string(),
            end: end.to_string(),
            increment: self.iterate_step,
            mode: self.iterate_by,
        }
    }

    fn get_between(&self) -> (&str, &str) {
        if self.iterate_between.is_empty() {
            (EMPTY_STR, EMPTY_STR)
        } else {
            (self.iterate_between[0].as_str(), self.iterate_between[1].as_str())
        }
    }
}

// endregion

#[cfg(test)]
mod tests {
    // region mockup

    use crate::{
        args::{Backup, Cli, Commands, IterateMode, shared::TEST_SELECT_FIELDS},
        fails::{BoxedResult, raise},
        helpers::{COMMA, EMPTY_STR},
        steps::{Slices, SolrCore},
    };
    use pretty_assertions::assert_eq;

    impl Commands {
        pub(crate) fn get(&self) -> BoxedResult<&Backup> {
            match &self {
                Self::Backup(gets) => Ok(&gets),
                _ => raise("command must be 'backup' !"),
            }
        }
    }

    impl SolrCore {
        pub(crate) fn mockup() -> Self {
            SolrCore { num_found: 100, fields: vec![TEST_SELECT_FIELDS.split(COMMA).collect()] }
        }
    }

    // endregion

    // region iterators

    #[test]
    fn check_iterator_for_params_get() {
        let parsed = Cli::mockup_args_backup();
        let gets = parsed.get().unwrap();
        let core_info = SolrCore::mockup();
        let query = gets.get_query_url(EMPTY_STR, true);

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
    fn check_iterator_for_slices_u64() {
        let slices = Slices::<String>::get_slice_of(16, 2);
        for step in slices {
            assert!(step.begin < step.end, "# {} -> {}", step.begin, step.end)
        }
    }

    #[test]
    fn check_iterator_for_slices_datetime() {
        let src = Slices::<String> {
            curr: "2020-04-01".to_string(),
            end: "2020-04-03T11:12:13".to_string(),
            increment: 1,
            mode: IterateMode::Day,
        };

        let slices = src.get_period_slices();
        assert!(slices.is_ok());

        if let Ok(seq) = slices {
            for step in seq {
                assert!(step.begin < step.end, "# {} -> {}", step.begin, step.end)
            }
        }
    }

    // endregion
}
