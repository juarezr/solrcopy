#![allow(dead_code)]

use log::{error, trace};
use zip::ZipArchive;

use glob::{glob, PatternError};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::args::Restore;
use crate::connection::http_post_as_json;
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

type Decompressor = ZipArchive<File>;

pub fn load_all_archives_for(archives_found: Vec<PathBuf>) -> impl Iterator<Item = ArchiveReader> {
    archives_found
        .into_iter()
        // .inspect(|archive_path| debug!("loading archive: {:?}", archive_path))
        .inspect(trace_archive)
        .flat_map(open_archive)
}

fn trace_archive(archive_path: &PathBuf) {
    trace!("loading archive: {:?}", archive_path)
}

fn open_archive(archive_path: PathBuf) -> Option<ArchiveReader> {
    let can_open = File::open(&archive_path);

    match can_open {
        Err(cause1) => {
            error!("error opening file: {:?} -> {}", archive_path, cause1);
            None
        }
        Ok(zipfile) => {
            let reader = ZipArchive::new(zipfile);
            match reader {
                Err(cause2) => {
                    error!("error loading archive: {:?} -> {}", archive_path, cause2);
                    None
                }
                Ok(archive) => Some(ArchiveReader {
                    archive,
                    entry_index: 0,
                }),
            }
        }
    }
}

#[derive(Debug)]
pub struct ArchiveReader {
    pub archive: Decompressor,
    pub entry_index: usize,
}

impl Iterator for ArchiveReader {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let file_count = self.archive.len();
        if self.entry_index >= file_count {
            return None;
        }
        let mut compressed = self.archive.by_index(self.entry_index).unwrap();
        let mut buffer = String::new();
        let reading = compressed.read_to_string(&mut buffer);
        match reading {
            Err(cause) => {
                error!("error reading archive #{}: {}", self.entry_index + 1, cause);
                None
            }
            Ok(_) => {
                self.entry_index += 1;
                Some(buffer)
            }
        }
    }
}

pub fn read_all_documents<T>(archives: T) -> impl Iterator<Item = String>
where
    T: Iterator<Item = ArchiveReader> + Sized,
{
    archives.inspect(trace_document).flatten()
}

fn trace_document(reader: &ArchiveReader) {
    trace!("reading document: {:?}", reader.entry_index)
}

pub fn post_content(update_hadler_url: &str, content: String) -> Option<()> {
    // TODO: handle network error, timeout on posting

    let response = http_post_as_json(&update_hadler_url, content);
    match response {
        Err(cause) => {
            error!("error updating solr at: {} -> {}", update_hadler_url, cause);
            None
        }
        Ok(_) => Some(()),
    }
}

// end of the file \\
