
use super::helpers::*;
use super::args::*;

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
        if self.limit <= self.curr { 
            None 
        } else {
            let remaining = self.limit - self.curr;
            let rows = self.batch.min(remaining);

            let query = format!("{}&start={}&rows={}", self.url, self.curr, rows);
            let res = Step {
                url: query,
                curr: self.curr
            };

            self.curr += self.batch;
            Some(res)
        }
    }
}

impl Steps {

    pub fn len(&self) -> u64 {
        let res = self.limit / self.batch;
        if self.limit % self.batch == 0 { res }
        else { res + 1 }
    }

    pub fn with_progress(self) -> ProgressBarIter<Steps> {
        let num_found = self.len() - 1;
        progress_with(self, num_found)
    }    
}

// endregion 

// region Solr Uri 

impl Arguments {

    pub fn get_steps(&self, core_info: &SolrCore) -> Steps {

        let gets = self.get().unwrap();

        let fl = Self::get_query_fields(&core_info.fields);
        let query = gets.get_query_url(self.url.as_ref(), fl);
        let rows = core_info.num_found.min(gets.limit.unwrap_or(std::u64::MAX));

        Steps {
            curr: 0,
            limit : rows,
            batch: gets.batch,
            url : query
        }
    }
 
    pub fn get_query_url(&self, selected: String) -> String {

        let gets = self.get().unwrap();

        gets.get_query_url(self.url.as_ref(), selected)
    }

    pub fn get_query_for_diagnostics(&self) -> String {

        let url = self.get_query_url(EMPTY_STRING);
        format!("{}&start=0&rows=1", url)
    }

    pub fn get_query_fields(selected: &Vec<String>) -> String {

        if selected.is_empty() { 
            EMPTY_STRING
        } else {
            let all = selected.join(COMMA);
            "&fl=".append(&all)
        }
   }
}

impl Get {

    pub fn get_query_url(&self, solr_url: &str, selected: String) -> String {

        let query = self.filter.clone()
            .unwrap_or("*:*".to_string())
            .replace(" or ", " OR ")
            .replace(" and ", " AND ")
            .replace(" not ", " NOT ")
            .replace(" ", "%20");

        let sort: String = if self.order.len() <= 0 { 
            EMPTY_STRING
        } else {
            let all: Vec<String> = self.order.iter()
                .map(|field| field.to_string())
                .collect();
            let joined = all.join(COMMA);
            "&sort=".append(&joined)
        };

        let parts: Vec<String> = vec!(
            solr_url.with_suffix("/"),
            self.from.clone(),
            "/select?wt=json&indent=off&omitHeader=true".to_string(),
            format!("&q={}", query),
            sort,
            selected,
        );
        parts.concat()
    }
}

// region ProgressBar

use indicatif::{ProgressBar, ProgressBarIter, ProgressIterator};

pub fn progress_with<S, It: Iterator<Item = S>>(steps: It, total: u64) -> ProgressBarIter<It> {

    let bar_style = indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:40.cyan/blue}] {pos}/{len}  {percent}% ({eta})");

    let bar = ProgressBar::new(total)
        .with_style(bar_style);
    
    let progr = steps.into_iter().progress_with(bar);
    progr
}

// endregion 

// endregion 

#[cfg(test)]
mod test {
    use crate::args::Arguments;
    use crate::args::SolrCore;
    use crate::helpers::*;

    #[test]
    fn check_iterator_for_params_get() {

        let parsed = Arguments::mockup_args_get();

        let core_info = SolrCore::mockup();

        let query = parsed.get_query_url(EMPTY_STRING);

        let mut i = 0;
        for step in parsed.get_steps(&core_info) {

            let url = step.url;
            assert_eq!(url.is_empty(), false);
            assert_eq!(url.starts_with(&query), true);

            i += 1;
        }    
        assert_eq!(i, 9);
    }

}

