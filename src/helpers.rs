#![allow(dead_code)]

use regex::Regex;

pub const EMPTY_STR: &'static str = "";
pub const EMPTY_STRING: String = String::new();

pub const COMMA: &'static str = ",";

pub trait StringHelpers {

    fn contains_any(&self, patterns: &[&str]) -> bool;

    fn starts_with_any(&self, patterns: &[&str]) -> bool;

    fn ends_with_any(&self, patterns: &[&str]) -> bool;

    fn append(&self, suffix: &str) -> String;

    fn append_all(&self, prefix: &[&str]) -> String;

    fn with_prefix(&self, prefix: &str) -> String;

    fn with_suffix(&self, suffix: &str) -> String;
}

impl StringHelpers for str {
    
    fn contains_any(&self, patterns: &[&str]) -> bool {
        for arg in patterns {
            if self.contains(arg) {
                return true;
            }
        }
        false
    }    
    
    fn starts_with_any(&self, patterns: &[&str]) -> bool {
        for arg in patterns {
            if self.starts_with(arg) {
                return true;
            }
        }
        false
    }    
    
    fn ends_with_any(&self, patterns: &[&str]) -> bool {
        for arg in patterns {
            if self.ends_with(arg) {
                return true;
            }
        }
        false
    }

    fn append(&self, suffix: &str) -> String {
        let mut res = String::with_capacity(self.len() + suffix.len());
        res.push_str(self);
        res.push_str(suffix);
        return res;
    }

    fn append_all(&self, suffixes: &[&str]) -> String {
        let mut all: Vec<&str> = Vec::with_capacity(suffixes.len() + 1);
        all.push(&self);
        all.extend(suffixes.iter());
        let res = all.concat();
        return res;
    }

    fn with_prefix(&self, prefix: &str) -> String {
        if self.starts_with(prefix) { 
            return self.to_string();
        } 
        let mut res = prefix.to_owned();
        res.push_str(&self);
        return res;
    }

    fn with_suffix(&self, suffix: &str) -> String {
        if self.ends_with(suffix) { 
            return self.to_string();
        } 
        let mut res = String::with_capacity(self.len() + suffix.len());
        res.push_str(self);
        res.push_str(suffix);
        return res;
    }    
}

pub trait RegexHelpers {

    fn get_group<'a>(&'a self, json: &'a str, group_number: usize) -> Option<&'a str>;

    fn get_groups<'a>(&self, json: &'a str, group_number: usize) -> Vec<&'a str>;

    fn get_matches<'a>(&self, json: &'a str) -> Vec<&'a str>;

    fn get_match_values(&self, json: &str) -> Vec<String>;
}

impl RegexHelpers for Regex {

    fn get_group<'a>(&self, json: &'a str, group_number: usize) -> Option<&'a str> {
        
        let mut matches = self.captures_iter(json);
        let group = matches.next();
        match group {
            None => None,
            Some(cap) => {
                match cap.get(group_number) {
                    None => None,
                    Some(group_text) => Some(group_text.as_str()),
                }
            }
        }
    }
    
    fn get_groups<'a>(&self, json: &'a str, group_number: usize) -> Vec<&'a str> {

        let matches = self.captures_iter(json);
        let caps = matches.map(|cap| { cap.get(group_number) } );
        let filt = caps.filter(|opt| {  opt.is_some() } );
        let maps = filt.map(|opt| { opt.unwrap().as_str() } );
        let res = maps.collect::<Vec<_>>();
        res
    }    
    
    fn get_matches<'a>(&self, json: &'a str) -> Vec<&'a str> {

        let matches = self.find_iter(json);
        let maps = matches.map(|m| { m.as_str() } );
        let res = maps.collect::<Vec<_>>();
        res
    }    
    
    fn get_match_values(&self, json: &str) -> Vec<String> {

        let matches = self.find_iter(json);
        let maps = matches.map(|m| { m.as_str().to_string() } );
        let res = maps.collect::<Vec<_>>();
        res
    }    
}

#[cfg(test)]
mod tests {
    use crate::helpers::*;

    #[test]
    fn check_starts_with_any() {
        let ok = &["true", "test"];
        let s1: &str = "test";
        assert_eq!(s1.starts_with_any(ok), true);
        let s2 : String = String::from("test");
        assert_eq!(s2.starts_with_any(ok), true);
    }
}