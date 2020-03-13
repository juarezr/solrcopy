#![allow(dead_code)]

use regex::{Captures, Regex};
use std::convert::TryInto;

// region Constants

pub const EMPTY_STR: &str = "";
pub const EMPTY_STRING: String = String::new();

pub const PT: &str = ".";
pub const COMMA: &str = ",";
pub const SEMICOLON: &str = ";";
pub const BRACKETS: &str = "{}";

pub const SPACE: char = ' ';
pub const ZERO: char = '0';

// endregion

// region Utility helpers

pub fn wait(secs: u64) {
    let millis = secs * 1000;
    std::thread::sleep(std::time::Duration::from_millis(millis));
}

// endregion

// region Type Method Extensions

pub trait StringHelpers {
    fn contains_any(&self, patterns: &[&str]) -> bool;

    fn starts_with_any(&self, patterns: &[&str]) -> bool;

    fn ends_with_any(&self, patterns: &[&str]) -> bool;

    fn append(&self, suffix: &str) -> String;

    fn append_all(&self, prefix: &[&str]) -> String;

    fn with_prefix(&self, prefix: &str) -> String;

    fn with_suffix(&self, suffix: &str) -> String;

    fn pad(&self, pad: usize) -> String;

    fn pad_0(&self, pad: usize) -> String;

    fn pad_with(&self, pad: usize, padchar: char) -> String;

    fn pad_left(&self, pad: usize) -> String;

    fn pad_0_left(&self, pad: usize) -> String;

    fn pad_left_with(&self, pad: usize, padchar: char) -> String;
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
        res
    }

    fn append_all(&self, suffixes: &[&str]) -> String {
        let mut all: Vec<&str> = Vec::with_capacity(suffixes.len() + 1);
        all.push(&self);
        all.extend(suffixes.iter());
        all.concat()
    }

    fn with_prefix(&self, prefix: &str) -> String {
        if self.starts_with(prefix) {
            return self.to_string();
        }
        let mut res = prefix.to_owned();
        res.push_str(&self);
        res
    }

    fn with_suffix(&self, suffix: &str) -> String {
        if self.ends_with(suffix) {
            return self.to_string();
        }
        let mut res = String::with_capacity(self.len() + suffix.len());
        res.push_str(self);
        res.push_str(suffix);
        res
    }

    fn pad(&self, pad: usize) -> String {
        Self::pad_left_with(self, pad, SPACE)
    }

    fn pad_0(&self, pad: usize) -> String {
        Self::pad_left_with(self, pad, ZERO)
    }

    fn pad_with(&self, pad: usize, padchar: char) -> String {
        let mut out = self.to_string();
        let len = self.len();
        let pad_len = pad - len;

        if pad_len > 0 {
            for _ in 0..pad_len {
                out.push(padchar);
            }
        }
        out
    }

    fn pad_left(&self, pad: usize) -> String {
        Self::pad_left_with(self, pad, SPACE)
    }

    fn pad_0_left(&self, pad: usize) -> String {
        Self::pad_left_with(self, pad, ZERO)
    }

    fn pad_left_with(&self, pad: usize, padchar: char) -> String {
        let mut out = String::new();
        let len = self.len();
        let pad_len = pad - len;

        if pad_len > 0 {
            for _ in 0..pad_len {
                out.push(padchar);
            }
        }
        out.push_str(self);
        out
    }
}

pub trait RegexHelpers {
    fn get_group<'a>(&'a self, json: &'a str, group_number: usize) -> Option<&'a str>;

    fn get_groups<'a>(&self, json: &'a str) -> Option<Captures<'a>>;

    fn get_group_values<'a>(&self, json: &'a str, group_number: usize) -> Vec<&'a str>;

    fn get_matches<'a>(&self, json: &'a str) -> Vec<&'a str>;

    fn get_match_values(&self, json: &str) -> Vec<String>;
}

impl RegexHelpers for Regex {
    fn get_group<'a>(&self, json: &'a str, group_number: usize) -> Option<&'a str> {
        let mut matches = self.captures_iter(json);
        let group = matches.next();
        match group {
            None => None,
            Some(cap) => match cap.get(group_number) {
                None => None,
                Some(group_text) => Some(group_text.as_str()),
            },
        }
    }

    fn get_groups<'a>(&self, json: &'a str) -> Option<Captures<'a>> {
        let mut matches = self.captures_iter(json);
        matches.next()
    }

    fn get_group_values<'a>(&self, json: &'a str, group_number: usize) -> Vec<&'a str> {
        let matches = self.captures_iter(json);
        let caps = matches.map(|cap| cap.get(group_number));
        let filt = caps.filter(|opt| opt.is_some());
        let maps = filt.map(|opt| opt.unwrap().as_str());
        maps.collect::<Vec<_>>()
    }

    fn get_matches<'a>(&self, json: &'a str) -> Vec<&'a str> {
        let matches = self.find_iter(json);
        let maps = matches.map(|m| m.as_str());
        maps.collect::<Vec<_>>()
    }

    fn get_match_values(&self, json: &str) -> Vec<String> {
        let matches = self.find_iter(json);
        let maps = matches.map(|m| m.as_str().to_string());
        maps.collect::<Vec<_>>()
    }
}

pub trait CapturesHelpers {
    /// Returns the match associated with the capture group at index `i`. If
    /// `i` does not correspond to a capture group, or if the capture group
    /// did not participate in the match, then a empty string is returned.
    ///
    /// # Examples
    ///
    /// Get the text of the match with a default of an empty string if this
    /// group didn't participate in the match:
    ///
    /// ```rust
    /// # use regex::Regex;
    /// let re = Regex::new(r"[a-z]+(?:([0-9]+)|([A-Z]+))").unwrap();
    /// let caps = re.captures("abc123").unwrap();
    ///
    /// let text1 = caps.get_as_str(1);
    /// let text2 = caps.get_as_str(2);
    /// assert_eq!(text1, "123");
    /// assert_eq!(text2, "");
    /// ```
    fn get_as_str(&self, i: usize) -> &str;

    /// Returns the match associated with the capture group at index `i`. If
    /// `i` does not correspond to a capture group, or if the capture group
    /// did not participate in the match, then a empty string is returned.
    ///
    /// # Examples
    ///
    /// Get the text of the match with a default of an empty string if this
    /// group didn't participate in the match:
    ///
    /// ```rust
    /// # use regex::Regex;
    /// let re = Regex::new(r"[a-z]+(?:([0-9]+)|([A-Z]+))").unwrap();
    /// let caps = re.captures("abc123").unwrap();
    ///
    /// let text1 = caps.get_as_str_or(1, "");
    /// let text2 = caps.get_as_str(2, "321");
    /// assert_eq!(text1, "123");
    /// assert_eq!(text2, "321");
    /// ```
    fn get_as_str_or<'a>(&'a self, i: usize, replacement: &'a str) -> &'a str;
}

impl CapturesHelpers for Captures<'_> {
    fn get_as_str(&self, i: usize) -> &str {
        self.get(i).map_or(EMPTY_STR, |m| m.as_str())
    }

    fn get_as_str_or<'a>(&'a self, i: usize, replacement: &'a str) -> &'a str {
        self.get(i).map_or(replacement, |m| m.as_str())
    }
}

// endregion

// region Numbers helpers

// TODO: investigate traits Convert From, etc..

pub trait SizeHelpers {
    fn to_u64(self) -> u64;

    fn to_usize(self) -> usize;
}

impl SizeHelpers for usize {
    fn to_u64(self) -> u64 {
        self.try_into().unwrap()
    }

    fn to_usize(self) -> usize {
        self
    }
}

impl SizeHelpers for u64 {
    fn to_u64(self) -> u64 {
        self
    }

    fn to_usize(self) -> usize {
        self.try_into().unwrap()
    }
}

// endregion

// region Debug helpers

pub(crate) fn print_env_vars() {
    eprintln!("Listing all env vars:");
    for (key, val) in std::env::vars() {
        eprintln!("  {}: {}", key, val);
    }
}

// endregion

#[cfg(test)]
mod tests {
    use crate::helpers::*;

    #[test]
    fn check_starts_with_any() {
        let ok = &["true", "test"];
        let s1: &str = "test";
        assert_eq!(s1.starts_with_any(ok), true);
        let s2: String = String::from("test");
        assert_eq!(s2.starts_with_any(ok), true);
    }
}
