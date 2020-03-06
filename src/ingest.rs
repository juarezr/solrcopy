use glob::{glob, PatternError};
use std::path::PathBuf;

use crate::args::Restore;
use crate::helpers::*;

impl Restore {
    pub fn find_archives(&self) -> Result<Vec<PathBuf>, PatternError> {
        let wilcard = self.get_pattern();
        let listed = glob(&wilcard)?;
        let found = listed.filter_map(Result::ok).collect::<Vec<_>>();
        Ok(found)
    }

    pub fn get_pattern(&self) -> String {
        let wilcard: String = match &self.pattern {
            Some(pat) => {
                if pat.ends_with(".zip") || pat.contains('*') {
                    pat.to_owned()
                } else {
                    format!("{}*", pat)
                }
            }
            None => format!("{}*.zip", self.into),
        };
        let mut path = self.from.clone();
        path.push(wilcard);
        let res = path.to_str().unwrap();
        res.to_string()
    }

    pub fn get_update_url(&self) -> String {
        let parts: Vec<String> = vec![
            self.options.url.with_suffix("/"),
            self.into.clone(),
            "/update".to_string(),
            self.commit.as_param("?"),
        ];
        parts.concat()
    }
}
