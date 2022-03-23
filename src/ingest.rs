use log::error;
use zip::ZipArchive;

use glob::{glob, PatternError};
use std::{fmt, fs::File, io::prelude::*, path::Path, path::PathBuf};

use crate::{
    args::{Restore, SortOrder},
    fails::*,
    helpers::*,
};

type Decompressor = ZipArchive<File>;

#[derive(Debug)]
pub(crate) struct ArchiveReader {
    pub archive: Decompressor,
    pub entry_index: usize,
}

pub(crate) struct Docs {
    pub json: String,
    pub archive: String,
    pub entry: String,
}

impl Restore {
    pub fn find_archives(&self) -> Result<Vec<PathBuf>, PatternError> {
        let wilcard = self.get_pattern();
        let listed = glob(&wilcard)?;
        let mut found = listed.filter_map(Result::ok).collect::<Vec<_>>();
        if self.order != SortOrder::None {
            found.sort_unstable();
        }
        if self.order == SortOrder::Desc {
            found.reverse();
        }
        Ok(found)
    }

    pub fn get_pattern(&self) -> String {
        let wilcard: String = match &self.search {
            Some(pat) => {
                if pat.ends_with(".zip") || pat.contains('*') {
                    pat.to_owned()
                } else {
                    format!("{}*", pat)
                }
            }
            None => format!("{}*.zip", self.options.core),
        };
        let mut path = self.transfer.dir.clone();
        path.push(wilcard);
        let res = path.to_str().unwrap();
        res.to_string()
    }

    pub fn get_update_url(&self) -> String {
        // E.g: http://localhost:8983/solr/mycore/update?wt=json&overwrite=true&commitWithin=1000&useParams=my_params
        let parts: Vec<String> = vec![
            self.options.get_core_handler_url("/update/json/docs?overwrite=true"),
            self.flush.as_param("&"),
            self.transfer.get_param("&"),
        ];
        parts.concat()
    }
}

impl ArchiveReader {
    pub(crate) fn open_archive(archive_path: &Path) -> BoxedResult<Decompressor> {
        let zipfile = File::open(archive_path)?;
        let res = ZipArchive::new(zipfile)?;
        Ok(res)
    }

    pub(crate) fn create_reader(archive_path: &Path) -> BoxedResult<ArchiveReader> {
        let success = Self::open_archive(archive_path);
        match success {
            Err(cause) => Err(cause),
            Ok(zip) => Ok(ArchiveReader { archive: zip, entry_index: 0 }),
        }
    }

    pub(crate) fn get_archive_file_count(archive_path: &Path) -> Option<usize> {
        let success = Self::open_archive(archive_path);
        match success {
            Err(_) => None,
            Ok(archive) => {
                let file_count = archive.len();
                Some(file_count)
            }
        }
    }
}

impl Iterator for ArchiveReader {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        let file_count = self.archive.len();
        if self.entry_index >= file_count {
            return None;
        }
        let mut compressed = self.archive.by_index(self.entry_index).unwrap();
        let zip_name = compressed.name().to_string();
        let mut zip_contents = String::new();
        let reading = compressed.read_to_string(&mut zip_contents);
        match reading {
            Err(cause) => {
                error!("error reading archive #{} {}: {}", self.entry_index + 1, zip_name, cause);
                None
            }
            Ok(_) => {
                self.entry_index += 1;
                Some((zip_name, zip_contents))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_steps = self.archive.len();
        if num_steps == 0 {
            (0, None)
        } else {
            let max: usize = num_steps.to_usize();
            (0, Some(max))
        }
    }
}

impl Docs {
    pub fn new(archive_name: String, entry_name: String, documents: String) -> Self {
        Docs { archive: archive_name, entry: entry_name, json: documents }
    }
}

impl fmt::Display for Docs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "archive: {} file: {}", self.archive, self.entry)
    }
}

impl fmt::Debug for Docs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let txt = if self.json.len() > 400 { &self.json[0..400] } else { &self.json };
        write!(f, "archive: {} file: {}\nJson: {}", self.archive, self.entry, txt)
    }
}

// end of the file \\
