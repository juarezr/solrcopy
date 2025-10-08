#![allow(dead_code)]

use regex::{Captures, Regex};
use std::{convert::TryInto, env, path::Path, str::FromStr};

// region Constants

pub(crate) const EMPTY_STR: &str = "";
pub(crate) const EMPTY_STRING: String = String::new();

pub(crate) const PT: &str = ".";
pub(crate) const COMMA: &str = ",";
pub(crate) const SEMICOLON: &str = ";";
pub(crate) const BRACKETS: &str = "{}";

pub(crate) const SPACE: char = ' ';
pub(crate) const ZERO: char = '0';

// endregion

// region Utility helpers

pub(crate) fn solr_query(query: &str) -> String {
    query
        .replace(" or ", " OR ")
        .replace(" and ", " AND ")
        .replace(" not ", " NOT ")
        .replace(' ', "%20")
}

const ISO_DATE: &str = "2020-01-01T00:00:00Z";
const ISO_SLEN: usize = 20;

pub(crate) fn replace_solr_date(query: &str, pattern: &str, value: &str) -> String {
    let vlen = value.len();
    let suffix = &ISO_DATE[vlen..];

    let value2 = value.append(suffix);
    query.replace(pattern, &value2)
}

pub(crate) fn wait(secs: usize) {
    let millis = secs * 1000;
    std::thread::sleep(std::time::Duration::from_millis(millis.to_u64()));
}

pub(crate) fn wait_by(millis: usize) {
    std::thread::sleep(std::time::Duration::from_millis(millis.to_u64()));
}

pub(crate) fn env_var(var_name: &str, replacement: &str) -> String {
    match env::var(var_name) {
        Ok(var_value) => var_value,
        Err(_) => replacement.to_string(),
    }
}

pub(crate) fn env_value(var_name: &str, replacement: isize) -> isize {
    match env::var(var_name) {
        Ok(var_value) => isize::from_str(&var_value).unwrap_or_else(|_| {
            panic!("Variable '{}' is not a integer value: {}", var_name, var_value)
        }),
        Err(_) => replacement,
    }
}

pub(crate) fn get_filename(file_path: &Path) -> Result<String, ()> {
    file_path.file_name().ok_or(())?.to_os_string().into_string().or(Err(()))
}

// endregion

// region Type Method Extensions

pub(crate) trait StringHelpers {
    fn contains_any(&self, patterns: &[&str]) -> bool;

    fn starts_with_any(&self, patterns: &[&str]) -> bool;

    fn ends_with_any(&self, patterns: &[&str]) -> bool;

    fn find_text_from<'a>(&'a self, text_to_search: &str, num_chars: isize) -> Option<&'a str>;

    fn find_text_between<'a>(&'a self, starts_text: &str, ends_text: &str) -> Option<&'a str>;

    fn append(&self, suffix: &str) -> String;

    fn append_all(&self, prefix: &[&str]) -> String;

    fn with_prefix(&self, prefix: &str) -> String;

    fn with_suffix(&self, suffix: &str) -> String;

    fn pad(&self, pad: usize) -> String;

    fn pad_0(&self, pad: usize) -> String;

    fn pad_with(&self, pad: usize, padchar: char) -> String;

    fn lpad(&self, pad: usize) -> String;

    fn lpad_0(&self, pad: usize) -> String;

    fn lpad_with(&self, pad: usize, padchar: char) -> String;

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
    fn find_text_from<'a>(&'a self, text_to_search: &str, num_chars: isize) -> Option<&'a str> {
        let (found, prefix) = self.match_indices(text_to_search).next()?;

        let starts = found + prefix.len();
        let text_len = self.len();

        let ulast_pos = num_chars.unsigned_abs();
        let positive = num_chars >= 0;
        let smaller = ulast_pos < text_len;

        let finish: usize = if positive && smaller {
            starts + ulast_pos
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
        Self::pad_with(self, pad, SPACE)
    }

    #[inline]
    fn pad_0(&self, pad: usize) -> String {
        Self::pad_with(self, pad, ZERO)
    }

    #[inline]
    fn pad_with(&self, pad: usize, padchar: char) -> String {
        let mut out = self.to_string();
        let len = self.len();
        let pad_len = pad as isize - len as isize;

        if pad_len > 0 {
            for _ in 0..pad_len {
                out.push(padchar);
            }
        }
        out
    }

    #[inline]
    fn lpad(&self, pad: usize) -> String {
        Self::lpad_with(self, pad, SPACE)
    }

    #[inline]
    fn lpad_0(&self, pad: usize) -> String {
        Self::lpad_with(self, pad, ZERO)
    }

    #[inline]
    fn lpad_with(&self, pad: usize, padchar: char) -> String {
        let mut out = String::new();
        let len = self.len();
        let pad_len = pad as isize - len as isize;

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
pub(crate) trait RegexHelpers {
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

pub(crate) trait CapturesHelpers {
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

pub(crate) trait IntegerHelpers {
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
mod test_utility_helpers {
    use crate::helpers::{
        env_value, env_var, get_filename, print_env_vars, replace_solr_date, solr_query, wait,
        wait_by,
    };
    use pretty_assertions::assert_eq;
    use std::path::Path;

    #[test]
    fn check_solr_query() {
        let query = "hello world";
        assert_eq!(solr_query(query), "hello%20world");
    }

    #[test]
    fn check_replace_solr_date() {
        let query = "started in {hello}";
        assert_eq!(replace_solr_date(query, "{hello}", "2025-01-01"), "started in 2025-01-01T00:00:00Z");
    }

    #[test]
    fn check_wait() {
        wait(1);
    }

    #[test]
    fn check_wait_by() {
        wait_by(1000);
    }

    #[test]
    fn check_env_var() {
        assert_eq!(env_var("TEST", "test"), "test");
    }

    #[test]
    fn check_env_value() {
        assert_eq!(env_value("TEST", 1), 1);
    }

    #[test]
    fn check_get_filename() {
        let path = Path::new("test.txt");
        assert_eq!(get_filename(path), Ok("test.txt".to_string()));
    }

    #[test]
    fn check_print_env_vars() {
        print_env_vars();
    }
}

#[cfg(test)]
mod test_string_helpers {
    use crate::helpers::StringHelpers;
    use pretty_assertions::assert_eq;

    #[test]
    fn check_starts_with_any() {
        let ok = &["true", "test"];
        let s1: &str = "test";
        assert_eq!(s1.starts_with_any(ok), true);
        let s2: String = String::from("test");
        assert_eq!(s2.starts_with_any(ok), true);
    }

    #[test]
    fn check_contains_any() {
        let patterns = &["foo", "bar", "baz"];
        let s1 = "hello foo world";
        assert_eq!(s1.contains_any(patterns), true);

        let s2 = "hello world";
        assert_eq!(s2.contains_any(patterns), false);

        let s3 = "barista";
        assert_eq!(s3.contains_any(patterns), true);

        let s4 = "";
        assert_eq!(s4.contains_any(patterns), false);

        let s5 = "bazooka";
        assert_eq!(s5.contains_any(patterns), true);
    }

    #[test]
    fn check_ends_with_any() {
        let patterns = &[".rs", ".txt", ".md"];
        let s1 = "file.rs";
        assert_eq!(s1.ends_with_any(patterns), true);

        let s2 = "document.pdf";
        assert_eq!(s2.ends_with_any(patterns), false);

        let s3 = "notes.md";
        assert_eq!(s3.ends_with_any(patterns), true);

        let s4 = "";
        assert_eq!(s4.ends_with_any(patterns), false);
    }

    #[test]
    fn check_find_text_from() {
        let s = "hello world, hello universe";
        // Find "hello" from after first
        assert_eq!(s.find_text_from("hello", 6), Some(" world"));
        // Not found
        assert_eq!(s.find_text_from("bye", 0), None);
        // Negative last_pos
        assert_eq!(s.find_text_from("world", -5), Some(", hello uni"));
        // last_pos beyond string
        assert_eq!(s.find_text_from("hello", 100), Some(" world, hello universe"));
        // Find "hello" from start
        assert_eq!(s.find_text_from("hello", 1), Some(" "));
    }

    #[test]
    fn check_find_text_between() {
        let s = "foo [bar] baz";
        assert_eq!(s.find_text_between("[", "]"), Some("bar"));
        let s2 = "no brackets here";
        assert_eq!(s2.find_text_between("[", "]"), None);
        let s3 = "[start] middle [end]";
        assert_eq!(s3.find_text_between("[", "]"), Some("start] middle [end"));
        let s4 = "prefix [content]";
        assert_eq!(s4.find_text_between("[", "]"), Some("content"));
    }

    #[test]
    fn check_append() {
        let s = "abc";
        assert_eq!(s.append("def"), "abcdef");
        let s2 = "";
        assert_eq!(s2.append("xyz"), "xyz");
        let s3 = "foo";
        assert_eq!(s3.append(""), "foo");
    }

    #[test]
    fn check_append_all() {
        let s = "x";
        let arr = &["a", "b", "c"];
        assert_eq!(s.append_all(arr), "xabc");
        let arr2: &[&str] = &[];
        assert_eq!(s.append_all(arr2), "x");
    }

    #[test]
    fn check_with_prefix() {
        let s = "bar";
        assert_eq!(s.with_prefix("foo"), "foobar");
        let s2 = "";
        assert_eq!(s2.with_prefix("pre"), "");
    }

    #[test]
    fn check_with_suffix() {
        let s = "foo";
        assert_eq!(s.with_suffix("bar"), "foobar");
        let s2 = "";
        assert_eq!(s2.with_suffix("suf"), "");
    }

    #[test]
    fn check_pad() {
        let s = "42";
        assert_eq!(s.pad(5), "42   ");
        let s2 = "hello";
        assert_eq!(s2.pad(3), "hello");
        let s3 = "";
        assert_eq!(s3.pad(4), "    ");
    }

    #[test]
    fn check_pad_0() {
        let s = "7";
        assert_eq!(s.pad_0(3), "700");
        let s2 = "abc";
        assert_eq!(s2.pad_0(2), "abc");
        let s3 = "";
        assert_eq!(s3.pad_0(2), "00");
    }

    #[test]
    fn check_pad_with() {
        let s = "hi";
        assert_eq!(s.pad_with(4, '_'), "hi__");
        let s2 = "test";
        assert_eq!(s2.pad_with(2, 'x'), "test");
        let s3 = "";
        assert_eq!(s3.pad_with(3, 'z'), "zzz");
    }

    #[test]
    fn check_pad_left() {
        let s = "42";
        assert_eq!(s.lpad(5), "   42");
        let s2 = "hello";
        assert_eq!(s2.lpad(3), "hello");
        let s3 = "";
        assert_eq!(s3.lpad(4), "    ");
    }

    #[test]
    fn check_pad_0_left() {
        let s = "7";
        assert_eq!(s.lpad_0(3), "007");
        let s2 = "abc";
        assert_eq!(s2.lpad_0(2), "abc");
        let s3 = "";
        assert_eq!(s3.lpad_0(2), "00");
    }

    #[test]
    fn check_pad_left_with() {
        let s = "hi";
        assert_eq!(s.lpad_with(4, '_'), "__hi");
        let s2 = "test";
        assert_eq!(s2.lpad_with(2, 'x'), "test");
        let s3 = "";
        assert_eq!(s3.lpad_with(3, 'z'), "zzz");
    }

    #[test]
    fn check_remove_whitespace() {
        let s = " a b c ";
        assert_eq!(s.remove_whitespace(), "abc");
        let s2 = "no_whitespace";
        assert_eq!(s2.remove_whitespace(), "no_whitespace");
        let s3 = "   ";
        assert_eq!(s3.remove_whitespace(), "");
        let s4 = "";
        assert_eq!(s4.remove_whitespace(), "");
        let s5 = "a\tb\nc";
        assert_eq!(s5.remove_whitespace(), "abc");
    }
}

#[cfg(test)]
mod test_regex_helpers {
    use crate::helpers::{CapturesHelpers, RegexHelpers};
    use pretty_assertions::assert_eq;
    use regex::Regex;

    // region Test RegexHelpers

    #[test]
    fn check_get_groups() {
        let re = Regex::new(r"(\d+)-(\d+)").unwrap();
        let cap = re.get_groups("123-456").unwrap();
        assert_eq!(cap.get_as_str(1), "123");
        assert_eq!(cap.get_as_str(2), "456");
    }

    #[test]
    fn check_get_group_values() {
        let re = Regex::new(r"(\d+)-(\d+)").unwrap();
        assert_eq!(re.get_group_values("123-456", 1), vec!["123"]);
        assert_eq!(re.get_group_values("123-456", 2), vec!["456"]);
    }

    #[test]
    fn check_get_matches() {
        let re = Regex::new(r"(\d+)-(\d+)").unwrap();
        assert_eq!(re.get_matches("123-456"), vec!["123-456"]);
    }

    #[test]
    fn check_get_match_values_single() {
        let re = Regex::new(r"(\d+)-(\d+)").unwrap();
        assert_eq!(re.get_match_values("123-456"), vec!["123-456"]);
    }

    #[test]
    fn check_get_match_values_multiple() {
        let re = Regex::new(r"(\d+)").unwrap();
        assert_eq!(re.get_match_values("123-456"), vec!["123", "456"]);
    }

    // endregion Test RegexHelpers

    // region Test CapturesHelpers

    #[test]
    fn check_get_as_str() {
        let re = Regex::new(r"(\d+)-(\d+)").unwrap();
        let caps = re.captures("123-456").unwrap();
        assert_eq!(caps.get_as_str(1), "123");
        assert_eq!(caps.get_as_str(2), "456");
    }

    #[test]
    fn check_get_as_str_or() {
        let re = Regex::new(r"(\d+)-(\d+)").unwrap();
        let caps = re.captures("123-456").unwrap();
        assert_eq!(caps.get_as_str_or(1, "0"), "123");
        assert_eq!(caps.get_as_str_or(2, "0"), "456");
    }

    #[test]
    fn check_get_as_str_or_empty() {
        let re = Regex::new(r"(\d+)-(\d+)").unwrap();
        let caps = re.captures("123-456").unwrap();
        assert_eq!(caps.get_as_str_or(1, "0"), "123");
        assert_eq!(caps.get_as_str_or(2, "0"), "456");
        assert_eq!(caps.get_as_str_or(3, "0"), "0");
    }

    // endregion Test CapturesHelpers
}

#[cfg(test)]
mod test_integer_helpers {
    use crate::helpers::IntegerHelpers;
    use pretty_assertions::assert_eq;

    #[test]
    fn check_to_u64_from_i64() {
        let i = 9876543210i64;
        assert_eq!(i.to_u64(), 9876543210u64);
    }

    #[test]
    fn check_to_usize_from_i64() {
        let i = 9876543210i64;
        assert_eq!(i.to_usize(), 9876543210usize);
    }

    #[test]
    fn check_to_isize_from_i64() {
        let i = 9876543210i64;
        assert_eq!(i.to_isize(), 9876543210isize);
    }

    #[test]
    fn check_to_i64_from_i64() {
        let i = 9876543210i64;
        assert_eq!(i.to_i64(), 9876543210i64);
    }

    #[test]
    fn check_to_i64_from_u64() {
        let i = 9876543210u64;
        assert_eq!(i.to_i64(), 9876543210i64);
    }

    #[test]
    fn check_to_usize_from_u64() {
        let i = 9876543210u64;
        assert_eq!(i.to_usize(), 9876543210usize);
    }

    #[test]
    fn check_to_isize_from_u64() {
        let i = 9876543210u64;
        assert_eq!(i.to_isize(), 9876543210isize);
    }

    #[test]
    fn check_to_u64_from_u64() {
        let i = 9876543210u64;
        assert_eq!(i.to_u64(), 9876543210u64);
    }

    #[test]
    fn check_to_u64_from_isize() {
        let i = 9876543210isize;
        assert_eq!(i.to_u64(), 9876543210u64);
    }

    #[test]
    fn check_to_i64_from_isize() {
        let i = 9876543210isize;
        assert_eq!(i.to_i64(), 9876543210i64);
    }

    #[test]
    fn check_to_usize_from_isize() {
        let i = 9876543210isize;
        assert_eq!(i.to_usize(), 9876543210usize);
    }

    #[test]
    fn check_to_isize_from_isize() {
        let i = 9876543210isize;
        assert_eq!(i.to_isize(), 9876543210isize);
    }

    #[test]
    fn check_to_u64_from_usize() {
        let i = 9876543210usize;
        assert_eq!(i.to_u64(), 9876543210u64);
    }

    #[test]
    fn check_to_i64_from_usize() {
        let i = 9876543210usize;
        assert_eq!(i.to_i64(), 9876543210i64);
    }

    #[test]
    fn check_to_isize_from_usize() {
        let i = 9876543210usize;
        assert_eq!(i.to_isize(), 9876543210isize);
    }

    #[test]
    fn check_to_usize_from_usize() {
        let i = 9876543210usize;
        assert_eq!(i.to_usize(), 9876543210usize);
    }
}
