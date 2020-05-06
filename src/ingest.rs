use log::error;
use zip::ZipArchive;

use glob::{glob, PatternError};
use std::{fs::File, io::prelude::*, path::PathBuf};

use crate::{args::Restore, fails::*, helpers::*};

type Decompressor = ZipArchive<File>;

impl Restore {
    pub fn find_archives(&self) -> Result<Vec<PathBuf>, PatternError> {
        let wilcard = self.get_pattern();
        let listed = glob(&wilcard)?;
        let found = listed.filter_map(Result::ok).collect::<Vec<_>>();
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
            None => format!("{}*.zip", self.into),
        };
        let mut path = self.from.clone();
        path.push(wilcard);
        let res = path.to_str().unwrap();
        res.to_string()
    }

    pub fn get_update_url(&self) -> String {
        // E.g: http://localhost:8983/solr/mycore/update?wt=json&overwrite=true&commitWithin=1000&useParams=my_params
        let parts: Vec<String> = vec![
            self.options.url.with_suffix("/"),
            self.into.clone(),
            "/update/json/docs?overwrite=true".to_string(),
            self.commit.as_param("&"),
            self.params.as_ref().unwrap_or(&EMPTY_STRING).with_prefix("&"),
        ];
        parts.concat()
    }
}

#[derive(Debug)]
pub(crate) struct ArchiveReader {
    pub archive: Decompressor,
    pub entry_index: usize,
}

impl ArchiveReader {
    pub(crate) fn open_archive(archive_path: &PathBuf) -> BoxedResult<Decompressor> {
        let zipfile = File::open(archive_path)?;
        let res = ZipArchive::new(zipfile)?;
        Ok(res)
    }

    pub(crate) fn create_reader(archive_path: &PathBuf) -> BoxedResult<ArchiveReader> {
        let success = Self::open_archive(archive_path);
        match success {
            Err(cause) => Err(cause),
            Ok(zip) => Ok(ArchiveReader { archive: zip, entry_index: 0 }),
        }
    }

    pub(crate) fn get_archive_file_count(archive_path: &PathBuf) -> Option<usize> {
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

// end of the file \\
