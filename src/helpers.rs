#![allow(dead_code)]

use regex::{Captures, Regex};
use std::{convert::TryInto, env, path::Path, str::FromStr};

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

pub fn solr_query(query: &str) -> String {
    query
        .replace(" or ", " OR ")
        .replace(" and ", " AND ")
        .replace(" not ", " NOT ")
        .replace(' ', "%20")
}

const ISO_DATE: &str = "2020-01-01T00:00:00Z";
const ISO_SLEN: usize = 20;

pub fn replace_solr_date(query: &str, pattern: &str, value: &str) -> String {
    let vlen = value.len();
    let suffix = &ISO_DATE[vlen..];

    let value2 = value.append(suffix);
    query.replace(pattern, &value2)
}

pub fn wait(secs: usize) {
    let millis = secs * 1000;
    std::thread::sleep(std::time::Duration::from_millis(millis.to_u64()));
}

pub fn wait_by(millis: usize) {
    std::thread::sleep(std::time::Duration::from_millis(millis.to_u64()));
}

pub fn env_var(var_name: &str, replacement: &str) -> String {
    match env::var(var_name) {
        Ok(var_value) => var_value,
        Err(_) => replacement.to_string(),
    }
}

pub fn env_value(var_name: &str, replacement: isize) -> isize {
    match env::var(var_name) {
        Ok(var_value) => isize::from_str(&var_value).unwrap_or_else(|_| {
            panic!("Variable '{}' is not a integer value: {}", var_name, var_value)
        }),
        Err(_) => replacement,
    }
}

pub fn get_filename(file_path: &Path) -> Result<String, ()> {
    file_path.file_name().ok_or(())?.to_os_string().into_string().or(Err(()))
}

// endregion

// region Type Method Extensions

pub trait StringHelpers {
    fn contains_any(&self, patterns: &[&str]) -> bool;

    fn starts_with_any(&self, patterns: &[&str]) -> bool;

    fn ends_with_any(&self, patterns: &[&str]) -> bool;

    fn find_text_from<'a>(&'a self, text_to_search: &str, last_pos: isize) -> Option<&'a str>;

    fn find_text_between<'a>(&'a self, starts_text: &str, ends_text: &str) -> Option<&'a str>;

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

    fn remove_whitespace(&self) -> String;
}

impl StringHelpers for str {
    #[inline]
    fn contains_any(&self, patterns: &[&str]) -> bool {
        for arg in patterns {
            if self.contains(arg) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn starts_with_any(&self, patterns: &[&str]) -> bool {
        for arg in patterns {
            if self.starts_with(arg) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn ends_with_any(&self, patterns: &[&str]) -> bool {
        for arg in patterns {
            if self.ends_with(arg) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn find_text_from<'a>(&'a self, text_to_search: &str, last_pos: isize) -> Option<&'a str> {
        let (found, prefix) = self.match_indices(text_to_search).next()?;

        let starts = found + prefix.len();
        let text_len = self.len();

        let ulast_pos = last_pos.abs() as usize;
        let positive = last_pos > 0;
        let smaller = ulast_pos < text_len;

        let finish: usize = if positive && smaller {
            ulast_pos
        } else if !positive && smaller {
            text_len - ulast_pos
        } else {
            text_len
        };
        if finish <= starts {
            return None;
        }
        let snippet = &self[starts..finish];
        Some(snippet)
    }

    fn find_text_between<'a>(&'a self, starts_text: &str, ends_text: &str) -> Option<&'a str> {
        let (start_pos, prefix) = self.match_indices(starts_text).next()?;
        let (ends_pos, _suffix) = self.rmatch_indices(ends_text).next()?;

        let starting = start_pos + prefix.len();
        if starting < ends_pos {
            let snippet = &self[starting..ends_pos];
            Some(snippet)
        } else {
            None
        }
    }

    #[inline]
    fn append(&self, suffix: &str) -> String {
        let mut res = String::with_capacity(self.len() + suffix.len());
        res.push_str(self);
        res.push_str(suffix);
        res
    }

    #[inline]
    fn append_all(&self, suffixes: &[&str]) -> String {
        let mut all: Vec<&str> = Vec::with_capacity(suffixes.len() + 1);
        all.push(self);
        all.extend(suffixes.iter());
        all.concat()
    }

    #[inline]
    fn with_prefix(&self, prefix: &str) -> String {
        if self.is_empty() || self.starts_with(prefix) {
            return self.to_string();
        }
        let mut res = prefix.to_owned();
        res.push_str(self);
        res
    }

    #[inline]
    fn with_suffix(&self, suffix: &str) -> String {
        if self.is_empty() || self.ends_with(suffix) {
            return self.to_string();
        }
        let mut res = String::with_capacity(self.len() + suffix.len());
        res.push_str(self);
        res.push_str(suffix);
        res
    }

    #[inline]
    fn pad(&self, pad: usize) -> String {
        Self::pad_left_with(self, pad, SPACE)
    }

    #[inline]
    fn pad_0(&self, pad: usize) -> String {
        Self::pad_left_with(self, pad, ZERO)
    }

    #[inline]
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

    #[inline]
    fn pad_left(&self, pad: usize) -> String {
        Self::pad_left_with(self, pad, SPACE)
    }

    #[inline]
    fn pad_0_left(&self, pad: usize) -> String {
        Self::pad_left_with(self, pad, ZERO)
    }

    #[inline]
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

    #[inline]
    fn remove_whitespace(&self) -> String {
        self.chars().filter(|c| !c.is_whitespace()).collect()
    }
}
pub trait RegexHelpers {
    fn get_group<'a>(&'a self, text_to_search: &'a str, group_number: usize) -> Option<&'a str>;

    fn get_groups<'a>(&self, text_to_search: &'a str) -> Option<Captures<'a>>;

    fn get_group_values<'a>(&self, text_to_search: &'a str, group_number: usize) -> Vec<&'a str>;

    fn get_matches<'a>(&self, text_to_search: &'a str) -> Vec<&'a str>;

    fn get_match_values(&self, text_to_search: &str) -> Vec<String>;
}

impl RegexHelpers for Regex {
    fn get_group<'a>(&self, text_to_search: &'a str, group_number: usize) -> Option<&'a str> {
        let mut matches = self.captures_iter(text_to_search);
        let group = matches.next();
        match group {
            None => None,
            Some(cap) => match cap.get(group_number) {
                None => None,
                Some(group_text) => Some(group_text.as_str()),
            },
        }
    }

    #[inline]
    fn get_groups<'a>(&self, text_to_search: &'a str) -> Option<Captures<'a>> {
        let mut matches = self.captures_iter(text_to_search);
        matches.next()
    }

    fn get_group_values<'a>(&self, text_to_search: &'a str, group_number: usize) -> Vec<&'a str> {
        let matches = self.captures_iter(text_to_search);
        let caps = matches.map(|cap| cap.get(group_number));
        let filt = caps.filter(|opt| opt.is_some());
        let maps = filt.map(|opt| opt.unwrap().as_str());
        maps.collect::<Vec<_>>()
    }

    fn get_matches<'a>(&self, text_to_search: &'a str) -> Vec<&'a str> {
        let matches = self.find_iter(text_to_search);
        let maps = matches.map(|m| m.as_str());
        maps.collect::<Vec<_>>()
    }

    fn get_match_values(&self, text_to_search: &str) -> Vec<String> {
        let matches = self.find_iter(text_to_search);
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
    #[inline]
    fn get_as_str(&self, i: usize) -> &str {
        self.get(i).map_or(EMPTY_STR, |m| m.as_str())
    }

    #[inline]
    fn get_as_str_or<'a>(&'a self, i: usize, replacement: &'a str) -> &'a str {
        self.get(i).map_or(replacement, |m| m.as_str())
    }
}

// endregion

// region Numbers helpers

pub trait IntegerHelpers {
    fn to_u64(self) -> u64;

    fn to_i64(self) -> i64;

    fn to_usize(self) -> usize;

    fn to_isize(self) -> isize;
}

impl IntegerHelpers for isize {
    #[inline]
    fn to_u64(self) -> u64 {
        self.try_into().unwrap()
    }

    #[inline]
    fn to_i64(self) -> i64 {
        self.try_into().unwrap()
    }

    #[inline]
    fn to_usize(self) -> usize {
        self.try_into().unwrap()
    }

    #[inline]
    fn to_isize(self) -> isize {
        self
    }
}

impl IntegerHelpers for usize {
    #[inline]
    fn to_u64(self) -> u64 {
        self.try_into().unwrap()
    }

    #[inline]
    fn to_i64(self) -> i64 {
        self.try_into().unwrap()
    }

    #[inline]
    fn to_usize(self) -> usize {
        self
    }

    #[inline]
    fn to_isize(self) -> isize {
        self.try_into().unwrap()
    }
}

impl IntegerHelpers for u64 {
    #[inline]
    fn to_u64(self) -> u64 {
        self
    }

    #[inline]
    fn to_i64(self) -> i64 {
        self.try_into().unwrap()
    }

    #[inline]
    fn to_usize(self) -> usize {
        self.try_into().unwrap()
    }

    #[inline]
    fn to_isize(self) -> isize {
        self.try_into().unwrap()
    }
}

impl IntegerHelpers for i64 {
    #[inline]
    fn to_u64(self) -> u64 {
        self.try_into().unwrap()
    }

    #[inline]
    fn to_i64(self) -> i64 {
        self
    }

    #[inline]
    fn to_usize(self) -> usize {
        self.try_into().unwrap()
    }

    #[inline]
    fn to_isize(self) -> isize {
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
